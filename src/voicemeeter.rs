//! Windows Remote API adapter. Uses the crate's existing bindings so that login
//! failures can be retried without the high-level wrapper's permanent logout.
use crate::{
    backend::{Backend, ChannelSnapshot},
    domain::Edition,
    settings::{Target, TargetKind},
};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    ffi::CString,
    time::{Duration, Instant},
};
use voicemeeter::bindings::VoicemeeterRemoteRaw;

#[derive(Default)]
pub struct Remote {
    raw: Option<&'static VoicemeeterRemoteRaw>,
    logged_in: bool,
    edition: Option<Edition>,
    pending: BTreeMap<String, Pending>,
}

struct Pending {
    value: f32,
    expires: Instant,
}

impl Pending {
    fn reconcile(&self, actual: f32, now: Instant) -> Option<f32> {
        (now < self.expires && (actual - self.value).abs() > 0.001).then_some(self.value)
    }
}

fn prefix(target: Target) -> String {
    format!(
        "{}[{}]",
        if target.kind == TargetKind::Strip {
            "Strip"
        } else {
            "Bus"
        },
        target.index
    )
}

fn level_channel(edition: Edition, target: Target, channel: usize) -> Result<(i32, i32)> {
    ensure!(edition.contains(target) && channel < 2, "targetOutOfRange");
    let (physical, _, _, _) = edition.counts();
    let index = usize::from(target.index);
    let offset = match target.kind {
        TargetKind::Bus => index * 8,
        TargetKind::Strip if target.index < physical => index * 2,
        TargetKind::Strip => usize::from(physical) * 2 + (index - usize::from(physical)) * 8,
    };
    Ok((
        if target.kind == TargetKind::Strip {
            1
        } else {
            3
        },
        (offset + channel) as i32,
    ))
}

fn status(code: i32, operation: &str) -> Result<()> {
    ensure!(code == 0, "{operation}: Remote API returned {code}");
    Ok(())
}

impl Remote {
    fn api(&self) -> Result<&'static VoicemeeterRemoteRaw> {
        ensure!(self.logged_in, "VoiceMeeter is not logged in");
        self.raw.context("VoiceMeeter DLL is not loaded")
    }

    fn check_target(&self, target: Target) -> Result<Edition> {
        let edition = self.edition.context("VoiceMeeter is not connected")?;
        ensure!(edition.contains(target), "targetOutOfRange");
        Ok(edition)
    }

    fn float(&mut self, name: &str) -> Result<f32> {
        let mut parameter = CString::new(name)?.into_bytes_with_nul();
        let mut value = 0.0;
        // SAFETY: the API is used only by its owning worker; pointers refer to
        // live, correctly sized buffers throughout this synchronous call.
        status(
            unsafe {
                self.api()?
                    .VBVMR_GetParameterFloat(parameter.as_mut_ptr().cast(), &mut value)
            },
            name,
        )?;
        ensure!(value.is_finite(), "{name}: non-finite value");
        Ok(self.reconcile(name, value))
    }

    fn reconcile(&mut self, name: &str, actual: f32) -> f32 {
        if let Some(value) = self
            .pending
            .get(name)
            .and_then(|p| p.reconcile(actual, Instant::now()))
        {
            return value;
        }
        self.pending.remove(name);
        actual
    }

    fn remember(&mut self, name: String, value: f32) {
        self.pending.insert(
            name,
            Pending {
                value,
                expires: Instant::now() + Duration::from_millis(500),
            },
        );
    }

    fn set_float(&mut self, name: &str, value: f32) -> Result<()> {
        ensure!(value.is_finite(), "non-finite parameter");
        let mut parameter = CString::new(name)?.into_bytes_with_nul();
        // SAFETY: owned NUL-terminated parameter buffer remains live during call.
        status(
            unsafe {
                self.api()?
                    .VBVMR_SetParameterFloat(parameter.as_mut_ptr().cast(), value)
            },
            name,
        )?;
        self.remember(name.into(), value);
        Ok(())
    }

    fn label(&self, target: Target) -> Result<String> {
        let mut parameter =
            CString::new(format!("{}.Label", prefix(target)))?.into_bytes_with_nul();
        // Remote API specifies a 512-character string buffer. Use its UTF-16
        // entry point, not the crate's ANSI helper, to preserve Japanese labels.
        let mut output = [0_u16; 512];
        // SAFETY: valid mutable NUL-terminated name and a 512-wide-character buffer.
        status(
            unsafe {
                self.api()?
                    .VBVMR_GetParameterStringW(parameter.as_mut_ptr().cast(), output.as_mut_ptr())
            },
            "read label",
        )?;
        let length = output
            .iter()
            .position(|c| *c == 0)
            .context("Label is not NUL terminated")?;
        String::from_utf16(&output[..length]).context("Invalid UTF-16 label")
    }

    fn level(&self, edition: Edition, target: Target, channel: usize) -> Result<f32> {
        let (kind, index) = level_channel(edition, target, channel)?;
        let mut amplitude = 0.0_f32;
        // SAFETY: validated channel index and live f32 output buffer.
        status(
            unsafe { self.api()?.VBVMR_GetLevel(kind, index, &mut amplitude) },
            "read level",
        )?;
        ensure!(amplitude.is_finite(), "non-finite level");
        Ok(if amplitude <= 0.0 {
            -120.0
        } else {
            20.0 * amplitude.log10()
        })
    }
}

impl Backend for Remote {
    fn connect(&mut self) -> Result<Option<Edition>> {
        // Native protocol smoke tests must never consume a real mixer session.
        if std::env::var_os("STREAMDECK_SMOKE_TEST").is_some_and(|value| value == "1") {
            return Ok(None);
        }
        self.pending.clear();
        if self.raw.is_none() {
            self.raw = Some(voicemeeter::get_voicemeeter_raw()?);
        }
        if !self.logged_in {
            // SAFETY: DLL is retained by the crate for the process lifetime;
            // this adapter is created and accessed on a single worker thread.
            let code = unsafe { self.raw.unwrap().VBVMR_Login() };
            tracing::info!(
                pid = std::process::id(),
                code,
                "VoiceMeeter login completed"
            );
            ensure!(code == 0 || code == 1, "VoiceMeeter login returned {code}");
            self.logged_in = true;
        }
        let mut kind = 0;
        // SAFETY: valid output pointer; only the worker accesses the session.
        let code = unsafe { self.api()?.VBVMR_GetVoicemeeterType(&mut kind) };
        if code == -2 {
            self.edition = None;
            return Ok(None);
        }
        status(code, "read edition")?;
        self.edition = match kind {
            1 => Some(Edition::Basic),
            2 => Some(Edition::Banana),
            3 | 6 => Some(Edition::Potato),
            0 => None,
            _ => anyhow::bail!("Unsupported VoiceMeeter type {kind}"),
        };
        Ok(self.edition)
    }

    fn poll(&mut self) -> Result<()> {
        // SAFETY: both dirty APIs are called by the sole owning worker.
        let code = unsafe { self.api()?.VBVMR_IsParametersDirty() };
        if code < 0 {
            self.edition = None;
            self.pending.clear();
            anyhow::bail!("Parameter polling returned {code}");
        }
        // MacroButtons may not be running even while the mixer is healthy.
        // Its availability is handled per Macro action by macro_state().
        let _ = unsafe { self.api()?.VBVMR_MacroButton_IsDirty() };
        Ok(())
    }

    fn channel(&mut self, target: Target) -> Result<ChannelSnapshot> {
        let edition = self.check_target(target)?;
        let name = prefix(target);
        let gain = self.float(&format!("{name}.Gain"))?;
        let mute = self.float(&format!("{name}.Mute"))? != 0.0;
        let mut flags = BTreeMap::new();
        let (physical, _, physical_buses, buses) = edition.counts();
        if target.kind == TargetKind::Strip {
            for i in 1..=physical_buses {
                let key = format!("A{i}");
                flags.insert(key.clone(), self.float(&format!("{name}.{key}"))? != 0.0);
            }
            for i in 1..=(buses - physical_buses) {
                let key = format!("B{i}");
                flags.insert(key.clone(), self.float(&format!("{name}.{key}"))? != 0.0);
            }
            flags.insert("Solo".into(), self.float(&format!("{name}.Solo"))? != 0.0);
            if target.index < physical {
                flags.insert("Mono".into(), self.float(&format!("{name}.Mono"))? != 0.0);
            } else if edition != Edition::Basic {
                flags.insert("MC".into(), self.float(&format!("{name}.MC"))? != 0.0);
                flags.insert("K".into(), self.float(&format!("{name}.Karaoke"))? != 0.0);
            }
            if target.index < physical && edition == Edition::Potato {
                flags.insert("EQ".into(), self.float(&format!("{name}.EQ.on"))? != 0.0);
            }
        } else if edition != Edition::Basic {
            flags.insert("EQ".into(), self.float(&format!("{name}.EQ.on"))? != 0.0);
            flags.insert("Mono".into(), self.float(&format!("{name}.Mono"))? != 0.0);
        }
        Ok(ChannelSnapshot {
            label: self.label(target)?,
            gain,
            mute,
            flags,
            levels: [
                self.level(edition, target, 0)?,
                self.level(edition, target, 1)?,
            ],
        })
    }

    fn set_gain(&mut self, target: Target, value: f32) -> Result<()> {
        self.check_target(target)?;
        ensure!((-60.0..=12.0).contains(&value), "gain out of range");
        self.set_float(&format!("{}.Gain", prefix(target)), value)
    }
    fn set_mute(&mut self, target: Target, value: bool) -> Result<()> {
        self.check_target(target)?;
        self.set_float(&format!("{}.Mute", prefix(target)), u8::from(value) as f32)
    }
    fn macro_state(&mut self, id: u8) -> Result<bool> {
        ensure!(id < 80, "macroOutOfRange");
        let mut value = 0.0;
        // SAFETY: validated ID, live output buffer, mode 0 is button state.
        status(
            unsafe {
                self.api()?
                    .VBVMR_MacroButton_GetStatus(i32::from(id), &mut value, 0)
            },
            "read macro",
        )?;
        Ok(self.reconcile(&format!("macro:{id}"), value) != 0.0)
    }
    fn set_macro(&mut self, id: u8, value: bool) -> Result<()> {
        ensure!(id < 80, "macroOutOfRange");
        let value = u8::from(value) as f32;
        // SAFETY: validated ID; default mode executes MacroButtons requests.
        status(
            unsafe {
                self.api()?
                    .VBVMR_MacroButton_SetStatus(i32::from(id), value, 0)
            },
            "write macro",
        )?;
        self.remember(format!("macro:{id}"), value);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        let mut macros: BTreeMap<u8, f32> = self
            .pending
            .iter()
            .filter_map(|(key, pending)| {
                key.strip_prefix("macro:")?
                    .parse()
                    .ok()
                    .map(|id| (id, pending.value))
            })
            .collect();
        if macros.is_empty() {
            return Ok(());
        }
        let deadline = Instant::now() + Duration::from_millis(500);
        while !macros.is_empty() {
            // SetStatus is asynchronous. Keep the DLL session alive and read
            // fresh status, bypassing our optimistic UI pending-value cache.
            std::thread::sleep(Duration::from_millis(10));
            self.poll()?; // Abort on mixer loss; never reconnect or resend here.
            let raw = self.api()?;
            // SAFETY: the worker owns the session throughout this bounded drain.
            let dirty = unsafe { raw.VBVMR_MacroButton_IsDirty() };
            ensure!(dirty >= 0, "Macro polling returned {dirty}");
            let mut confirmed = Vec::new();
            for (&id, &expected) in &macros {
                let mut actual = 0.0;
                // SAFETY: valid ID and output pointer; no concurrent API calls.
                status(
                    unsafe { raw.VBVMR_MacroButton_GetStatus(i32::from(id), &mut actual, 0) },
                    "confirm macro",
                )?;
                if (actual - expected).abs() < 0.001 {
                    confirmed.push(id);
                }
            }
            for id in confirmed {
                macros.remove(&id);
            }
            ensure!(
                macros.is_empty() || Instant::now() < deadline,
                "Macro write confirmation timed out"
            );
        }
        tracing::info!("Pending Macro writes confirmed before logout");
        Ok(())
    }
}

impl Drop for Remote {
    fn drop(&mut self) {
        if self.logged_in
            && let Some(raw) = self.raw
        {
            // SAFETY: the worker has stopped all polling and released held keys.
            let code = unsafe { raw.VBVMR_Logout() };
            tracing::info!(
                pid = std::process::id(),
                code,
                "VoiceMeeter logout completed"
            );
            if code != 0 {
                tracing::warn!(code, "VoiceMeeter logout failed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn level_offsets_cover_editions_and_virtual_channels() {
        for edition in [Edition::Basic, Edition::Banana, Edition::Potato] {
            let (physical, strips, _, buses) = edition.counts();
            assert_eq!(
                level_channel(
                    edition,
                    Target {
                        kind: TargetKind::Strip,
                        index: physical
                    },
                    1
                )
                .unwrap(),
                (1, i32::from(physical) * 2 + 1)
            );
            assert_eq!(
                level_channel(
                    edition,
                    Target {
                        kind: TargetKind::Bus,
                        index: buses - 1
                    },
                    1
                )
                .unwrap(),
                (3, i32::from(buses - 1) * 8 + 1)
            );
            assert!(
                level_channel(
                    edition,
                    Target {
                        kind: TargetKind::Strip,
                        index: strips
                    },
                    0
                )
                .is_err()
            );
        }
    }
    #[test]
    fn pending_writes_bridge_stale_reads_but_expire_and_accept_acknowledgement() {
        let now = Instant::now();
        let pending = Pending {
            value: 6.0,
            expires: now + Duration::from_millis(500),
        };
        assert_eq!(pending.reconcile(3.0, now), Some(6.0));
        assert_eq!(pending.reconcile(6.0, now), None);
        assert_eq!(pending.reconcile(-2.0, now + Duration::from_secs(1)), None);
    }
}
