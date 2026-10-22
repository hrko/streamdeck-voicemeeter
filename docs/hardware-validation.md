# Hardware acceptance record

Status: pending. Automated fake-backend and loopback tests do not constitute
hardware acceptance. Do not mark a row passed without observing its result.

## Initial observations (2026-10-23)

Installation succeeded, and the user supplied a screenshot showing the PI with
Potato connected. Gain/Combo touch-strip rendering remained blank both in the
application and on hardware. Explicit layout selection and SVG dimensions did
not resolve it after the installed EXE was hash-verified and Stream Deck restarted.
The user confirmed that display appeared after the single-viewport SVG revision
(`4f3d7f3`) was installed. Nested SVG viewports were replaced by group transforms;
the existing icon paths and settings remain intact. This confirms resolution of
the blank display, not every rendering/state/meter acceptance item below.

The user then confirmed all requested normal-operation checks: Gain rotation
with a 3 dB step and push/tap mute, independent Combo steps with normal/pressed
rotation and left/right tap mute, and Macro Toggle/Push with external-state
synchronization. These observations apply to Windows 11 / Stream Deck + /
Potato only. Limits, restart/lifecycle cases and visual details remain separate.

Further user checks confirmed offline PI operation, recovery after restarting
VoiceMeeter without replaying offline dial input, release of the original Push
target on page/target changes, shared-target display synchronization, state
flags and Japanese labels. The user separately confirmed all icon font detail
controls. Meter decay differed from the Go version: the Rust renderer lacked
its 12 dB/second filled falling peak and instantaneous-level marker. The fix
restores these with shared per-target history. The user confirmed the corrected
decay and current-level marker on Gain/Combo, including multiple meters for the
same target. Tested EXE from `04ab431` has SHA-256
`fd553dfdde4b24ce1f1ab1b3a1f4ee9c9ae6f929c3572d687f6d87720909f46b`.

## Available environment

### Development-update connection incident

After repeated development updates, the owner reported frozen meters, ineffective
controls and all eight Remote API indicators occupied. Only one Rust plugin
process remained. The earlier helper's force termination could bypass `Remote`
drop/logout; the indicators alone do not prove which process owned each slot.
The helper now signals a per-process Windows event and waits for normal exit
before stopping the host. It refuses to force-kill. Native smoke tests disable
real mixer login, and CI tests event-driven exit with the WebSocket still open.
The owner stopped VoiceMeeter for the one-time upgrade of the old binary.
The installed replacement (`be11243a2520d99d9aa4ed5029021ec803e7c2deae0908750b8301a24ce7e79b`)
then stopped gracefully through the helper and restarted successfully.
The owner confirmed that meters and gain/mute operations recovered after mixer
restart, but the full R-indicator display remained. Process start times confirmed
that the mixer restarted; MacroButtons remained running across that restart.
Readable process-module inspection found the Remote DLL only in the single Rust
plugin process (this is not a complete inventory of all possible clients).
The crate documents stale visual indicators when applications exit without logout;
we have not proven whether the remaining display represents live or stale slots.
The owner then closed both the mixer and MacroButtons. After graceful plugin
shutdown and mixer startup, Stream Deck automatically relaunched the plugin;
this invalidated the first proposed disconnected baseline. We gracefully stopped
the plugin again and closed the host. Process inspection then found only the
mixer, and the owner observed one R indicator. Starting Stream Deck produced
exactly one Rust plugin process and the owner observed two R indicators: one
additional indicator for this connection, no longer the full display. The origin
of the baseline indicator is unproven. MacroButtons was restored afterward.
Repeated-update connection-count stability remains unverified; do not infer it
from this single reconnect. Meter/gain/mute recovery was confirmed earlier above.

The owner confirmed `9b34802` removes the brief Macro slash on page changes.
They also reported four R indicators after MacroButtons restoration and that
development update; there was no recorded baseline after MacroButtons startup.
Inspection found a redundant CLI restart after host startup in the development
helper. This was removed: host startup now launches the plugin once, and the
helper only verifies that exactly one matching process appears. A subsequent
update with MacroButtons unchanged completed successfully; indicator comparison
against the four-indicator baseline failed: the owner observed five indicators.
PID/API diagnostics then showed logout success for process 78620 followed by
automatic host relaunch as 78936 about two seconds later. The host-close helper
waited three seconds before terminating the tray host, leaving enough time for
this relaunch. It now requests another graceful plugin stop immediately before
host termination. A subsequent update logged logout=0 for both the original
78936 and relaunched 71880 before the host stopped; startup created only 23608.
The diagnostic installation itself used the old helper, so its indicator baseline
may have increased once more. Final visual count/stability is still pending.

The subsequent Go appearance restoration (`fe5e769`, title defaults corrected
in `79bed2c`) was also checked by the owner. Icons, status indicators, gain
sliders and Macro icon size matched expectations. After repairing persisted
hidden-title preferences, Gain/left-Combo labels and Macro ON/OFF titles became
visible; the owner reported no apparent text size or position problem.
Installed EXE SHA-256:
`312f7b9e535e6122f11448882d2ce0ccc9770d04cd457e81ca936a803cb601e7`.
See appearance-parity.md for the separate official-layout comparison.

- User-confirmed: Windows 11, Stream Deck +, VoiceMeeter Potato.
- Installed Stream Deck executable: 7.0.3.22071 (file metadata).
- Installed Potato executable: 3.1.2.2 (file metadata).
- Windows release/build read from the registry: 26H2 / 26300.9711. The user
  identifies the OS as Windows 11 (the legacy ProductName registry value says
  Windows 10 Pro). Device firmware is not yet recorded.
- Basic and Banana executables exist locally, but neither edition has been
  confirmed as an available test environment. Windows 10 is also unverified.

## Before testing

Record the Git commit, package SHA-256, Windows build, Stream Deck version,
device firmware and VoiceMeeter edition/version below. Save the current mixer
configuration and use a dedicated test page with targets that will not interrupt
live audio. Install the package following usage.md, or use `mise run dev` for
the development link. The new plugin ID does not migrate existing settings.

| Field | Observed value |
| --- | --- |
| Commit / package SHA-256 | Installed EXE from `04ab431`, hash above; package was followed by an EXE replacement. |
| Windows build | Windows 11, 26H2, 26300.9711 |
| Stream Deck / firmware | App 7.0.3.22071; firmware not recorded |
| VoiceMeeter edition/version | Potato 3.1.2.2 |
| Install or development link | Package installation, then verified EXE replacement |
| Date / tester | 2026-10-23 JST / repository owner |

## Single-session procedure

For each row record pass/fail, observations and relevant logs. Restore mixer
settings after testing. In particular, an interrupted Push is deliberately not
released into a new VoiceMeeter session; inspect and reset its MacroButton state
manually when necessary.

| Check | Procedure and expected behavior | Result |
| --- | --- | --- |
| Installation | Install or dev-link; all three actions appear without replacing the old plugin. | Package installed; all three actions exercised. On 2026-10-23, packaged-to-development conversion and a repeated `mise run dev` succeeded; junction and running EXE verified. Release packaging also passed while the development link remained active. No separate UI acceptance of the debug build is claimed. |
| Offline PI | Open PI with VoiceMeeter stopped; forms and saved settings remain usable and status reports disconnected. | PI remains usable per user. |
| Startup recovery | Start Potato; status and displays recover without reloading the plugin. Offline operations are not replayed. | Passed per user, including no gain change from offline dial input. |
| Gain | Rotate slowly and rapidly with a 3 dB step; values accumulate and stop at -60/+12 dB. Press and tap toggle mute. | 3 dB rotation and press/tap mute passed per user; rapid input/limits not separately confirmed. |
| Combo | Set left step 3 and right step 0.5. Rotate normally and while pressed; each side uses its own target/step. Tap either half and the center boundary; only the expected target toggles. | Independent steps, normal/pressed rotation and left/right mute passed per user; exact center boundary not separately confirmed. |
| Shared targets | Put the same target on multiple actions. Operate either and change gain/mute in VoiceMeeter; all displays converge. | Synchronization passed per user. |
| Toggle | Press a Macro Toggle repeatedly, then change its state externally; state, icon and color synchronize. | Passed per user. |
| Push | Hold/release a Macro Push. Repeat while changing its target/settings, changing page and normally closing the plugin; original target is released when communication remains available. | Normal Push, page change and target change passed per user; normal close not separately confirmed. |
| Overlapping Push | Hold two actions pointing at one MacroButton; releasing the first keeps it ON, releasing the last turns it OFF. | Pending |
| Restart during Push | Restart VoiceMeeter during a hold. The old press/release must not be replayed in the new session; a fresh press/release works. | Pending |
| Invalid target | Configure an out-of-range Strip/Bus or Macro ID; show an error and never operate on target zero. | Pending |
| State and meters | Compare stereo meters, mute, solo, routing, Mono/MC and EQ against VoiceMeeter on representative physical/virtual strips and buses. | State synchronization passed. Corrected meter decay/current marker and shared-meter behavior passed per user on `04ab431`. Full target/flag matrix not individually recorded. |
| Rendering | Use long and non-ASCII labels, three icon styles and extreme supported axes. No cross-channel overlap; missing codepoints show a warning/fallback. | Japanese labels and font-detail controls passed per user. Long-label/extreme-axis/missing-codepoint cases not separately recorded. |
| PI | Restore settings, edit independent Combo values and Macro colors/icons, expand details, enter invalid numbers and navigate by keyboard. Invalid intermediate values are not saved. | Pending |
| PI appearance | Compare with a built-in PI at the same window size: colors, font size, label width, control height, spacing and focus. Record differences. | Pending |
| Localization | Switch host language between Japanese and English and restart the plugin. Confirm translated PI/action labels and unchanged saved targets. | Pending |
| Lifecycle/performance | Change pages repeatedly and operate multiple actions. No updates to disappeared actions, stale meter backlog or sustained excessive CPU usage. Record observed CPU and responsiveness. | Pending |
| Diagnostics | Confirm startup/errors/shutdown in the documented rotating log directory; attach relevant excerpts to failures. | Actual host WARN record observed in the documented directory; inherited `RUST_LOG=warn` suppresses INFO. Fresh isolated debug/release runs verify startup, normal shutdown and invalid-launch ERROR records. Normal host shutdown with INFO enabled is not separately recorded. |

## Coverage outside this machine

| Environment | Status |
| --- | --- |
| Windows 11 / Stream Deck + / Potato | Normal actions, recovery, Push page/target release, shared targets, state flags, Japanese labels and font controls passed; remaining cases above. |
| Windows 10 x64 | Not tested |
| VoiceMeeter Basic | Not tested |
| VoiceMeeter Banana | Not tested |
| Stream Deck 6.4 minimum version | Not tested; manifest declares this minimum |

Do not infer compatibility verification for these environments from a successful
Potato test. Record separate results when those environments become available.
