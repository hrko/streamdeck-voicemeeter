//! Per-process shutdown event for development updates without TerminateProcess.
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::Threading::{CreateEventW, WaitForSingleObject},
};

pub struct ShutdownEvent(usize);
impl ShutdownEvent {
    pub fn new() -> anyhow::Result<Self> {
        let name: Vec<u16> = format!(
            "Local\\streamdeck-voicemeeter-stop-{}\0",
            std::process::id()
        )
        .encode_utf16()
        .collect();
        // SAFETY: null security attributes select defaults; name is NUL terminated.
        let handle = unsafe { CreateEventW(std::ptr::null(), 1, 0, name.as_ptr()) };
        anyhow::ensure!(
            !handle.is_null(),
            "Cannot create shutdown event: {}",
            std::io::Error::last_os_error()
        );
        Ok(Self(handle as usize))
    }
    pub async fn requested(&self) {
        loop {
            // SAFETY: this object owns the handle until the future has finished.
            if unsafe { WaitForSingleObject(self.0 as _, 0) } == 0 {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }
}
impl Drop for ShutdownEvent {
    fn drop(&mut self) {
        // SAFETY: owned handle, closed exactly once after polling ends.
        unsafe {
            CloseHandle(self.0 as _);
        }
    }
}
