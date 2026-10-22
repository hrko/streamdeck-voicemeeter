use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

struct RotatingFile {
    directory: PathBuf,
    file: Option<File>,
    bytes: u64,
    limit: u64,
}
impl RotatingFile {
    fn open(directory: &Path, limit: u64) -> io::Result<Self> {
        fs::create_dir_all(directory)?;
        let directory = directory.canonicalize()?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join("plugin.log"))?;
        let bytes = file.metadata()?.len();
        Ok(Self {
            directory,
            file: Some(file),
            bytes,
            limit,
        })
    }
    fn rotate(&mut self) -> io::Result<()> {
        self.file.take();
        let current = self.directory.join("plugin.log");
        let previous = self.directory.join("plugin.1.log");
        let oldest = self.directory.join("plugin.2.log");
        // All rotation targets are fixed filenames inside the canonical log directory.
        for file in [&current, &previous, &oldest] {
            if file.parent() != Some(self.directory.as_path()) {
                return Err(io::Error::other("Invalid log path"));
            }
        }
        if oldest.exists() {
            fs::remove_file(&oldest)?;
        }
        if previous.exists() {
            fs::rename(&previous, &oldest)?;
        }
        if current.exists() {
            fs::rename(&current, &previous)?;
        }
        self.file = Some(OpenOptions::new().create(true).append(true).open(current)?);
        self.bytes = 0;
        Ok(())
    }
    fn append(&mut self, bytes: &[u8]) -> io::Result<()> {
        let retained = &bytes[..bytes.len().min(self.limit as usize)];
        if self.file.is_none() || self.bytes + retained.len() as u64 > self.limit {
            self.rotate()?;
        }
        self.file
            .as_mut()
            .ok_or_else(|| io::Error::other("Log unavailable"))?
            .write_all(retained)?;
        self.bytes += retained.len() as u64;
        Ok(())
    }
}

#[derive(Clone)]
struct Writer(Arc<Mutex<RotatingFile>>);
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("Log lock poisoned"))?
            .append(bytes)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = self
            .0
            .lock()
            .map_err(|_| io::Error::other("Log lock poisoned"))?
            .file
            .as_mut()
        {
            file.flush()?;
        }
        Ok(())
    }
}

pub fn initialize() -> anyhow::Result<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let directory = base.join(crate::actions::PLUGIN_ID).join("logs");
    let writer = Writer(Arc::new(Mutex::new(RotatingFile::open(
        &directory,
        1024 * 1024,
    )?)));
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_writer(move || writer.clone())
        .try_init()
        .map_err(|error| anyhow::anyhow!("Cannot initialize logging: {error}"))?;
    std::panic::set_hook(Box::new(|panic| {
        tracing::error!(%panic, "Plugin panicked");
    }));
    Ok(directory)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotation_caps_count_and_file_size_including_oversized_messages() {
        let directory =
            std::env::temp_dir().join(format!("voicemeeter-log-test-{}", std::process::id()));
        let mut file = RotatingFile::open(&directory, 64).unwrap();
        for _ in 0..10 {
            file.append(&[b'x'; 48]).unwrap();
        }
        file.append(&[b'y'; 1000]).unwrap();
        drop(file);
        let entries: Vec<_> = fs::read_dir(&directory)
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(entries.len(), 3);
        for entry in entries {
            assert!(entry.metadata().unwrap().len() <= 64);
            fs::remove_file(entry.path()).unwrap();
        }
        fs::remove_dir(directory).unwrap();
    }
}
