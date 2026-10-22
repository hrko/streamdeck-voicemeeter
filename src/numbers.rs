/// Format display-only gain; serialized settings and API values remain numeric.
pub fn gain(value: f32, locale: &str) -> String {
    if !value.is_finite() {
        return "—".into();
    }
    let invariant = format!("{value:.1}");
    #[cfg(windows)]
    if let Some(formatted) = windows_number(&invariant, locale) {
        return formatted;
    }
    #[cfg(not(windows))]
    let _ = locale;
    invariant
}

#[cfg(windows)]
fn windows_number(value: &str, locale: &str) -> Option<String> {
    use windows_sys::Win32::Globalization::{
        GetLocaleInfoEx, GetNumberFormatEx, LOCALE_INEGNUMBER, LOCALE_SDECIMAL, LOCALE_STHOUSAND,
        NUMBERFMTW, ResolveLocaleName,
    };
    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }
    if locale.contains('\0') || locale.is_empty() {
        return None;
    }
    let name = wide(&locale.replace('_', "-"));
    let mut resolved = [0u16; 85];
    // SAFETY: all input strings are NUL-terminated; output lengths match their
    // live buffers. No API retains pointers after returning.
    unsafe {
        if ResolveLocaleName(name.as_ptr(), resolved.as_mut_ptr(), resolved.len() as i32) == 0 {
            return None;
        }
        let info = |kind| {
            let mut buffer = [0u16; 32];
            let len = GetLocaleInfoEx(
                resolved.as_ptr(),
                kind,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
            );
            (len > 0).then_some(buffer)
        };
        let mut decimal = info(LOCALE_SDECIMAL)?;
        let mut thousands = info(LOCALE_STHOUSAND)?;
        let negative = info(LOCALE_INEGNUMBER)?;
        let end = negative.iter().position(|v| *v == 0)?;
        let order = String::from_utf16(&negative[..end]).ok()?.parse().ok()?;
        let format = NUMBERFMTW {
            NumDigits: 1,
            LeadingZero: 1,
            Grouping: 0,
            lpDecimalSep: decimal.as_mut_ptr(),
            lpThousandSep: thousands.as_mut_ptr(),
            NegativeOrder: order,
        };
        let input = wide(value);
        let mut output = [0u16; 128];
        let len = GetNumberFormatEx(
            resolved.as_ptr(),
            0,
            input.as_ptr(),
            &format,
            output.as_mut_ptr(),
            output.len() as i32,
        );
        if len <= 0 {
            return None;
        }
        String::from_utf16(&output[..len as usize - 1]).ok()
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn gain_uses_host_locale_with_safe_fallback() {
        assert_eq!(gain(-3.5, "ja"), "-3.5");
        assert_eq!(gain(-3.5, "en-US"), "-3.5");
        assert_eq!(gain(-3.5, "de-DE"), "-3,5");
        assert_eq!(gain(0.5, "fr_FR"), "0,5");
        assert_eq!(gain(0.5, "not-a-real-locale"), "0.5");
        assert_eq!(gain(0.5, "de\0-DE"), "0.5");
        assert_eq!(gain(f32::NAN, "en"), "—");
    }
}
