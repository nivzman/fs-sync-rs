use std::{fs::File, time::{Duration, SystemTime}, io::{Seek, SeekFrom}};
use std::io::{Read, Write};

pub use error::Error;
use header::Header;

mod header;
mod error;

const EXPIRATION_TIMEOUT: Duration = Duration::from_secs(120);
const HEADER_WRITE_SLEEP_TIME: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct Mutex {
    path: std::path::PathBuf,
}

#[derive(Debug)]
pub struct Guard {
    file: File,
}

impl Guard {
    pub fn read_contents(&mut self, buff: &mut Vec<u8>) -> Result<(), Error> {
        self.file.seek(SeekFrom::Start(header::HEADER_SIZE as u64))?;
        self.file.read_to_end(buff)?;
        Ok(())
    }

    pub fn write_contents(&mut self, buff: &[u8]) -> Result<(), Error> {
        self.file.seek(SeekFrom::Start(header::HEADER_SIZE as u64))?;
        self.file.write_all(buff)?;
        Ok(())
    }

    fn clear_header(&mut self) -> Result<(), Error> {
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(header::CLEAR_HEADER.as_ref())?;
        Ok(())
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        // Not much to do, the expiration mechanism would take care of the leak after the timeout
        let _ = self.clear_header();
    }
}

impl Mutex {
    pub fn new(path: &str) -> Self {
        Mutex {
            path: path.into(),
        }
    }

    pub fn try_lock(&self) -> Result<Option<Guard>, Error> {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.path)?;

        if Mutex::is_locked(&mut file)? {
           return Ok(None);
        }

        if !Mutex::try_write_header(&mut file)? {
            return Ok(None);
        }

        Ok(Some(Guard { file }))
    }

    fn is_locked(file: &mut File) -> Result<bool, Error> {
        Ok(!Mutex::is_expired(file)? && Mutex::check_header_integrity(file)?)
    }

    fn is_expired(file: &File) -> Result<bool, Error>  {
        let last_modify = file.metadata()?.modified()?;
        let duration_since_modify = SystemTime::now().duration_since(last_modify)?;
        Ok(duration_since_modify >= EXPIRATION_TIMEOUT)
    }

    fn check_header_integrity(file: &mut File) -> Result<bool, Error>  {
        let header = Mutex::read_header(file)?;
        Ok(header != header::CLEAR_HEADER && header::check_integrity(&header))
    }

    fn try_write_header(file: &mut File) -> Result<bool, Error> {
        let header = header::new();
        file.seek(SeekFrom::Start(0))?;
        file.write_all(header.as_slice())?;

        std::thread::sleep(HEADER_WRITE_SLEEP_TIME);

        Ok(Mutex::read_header(file)? == header)
    }

    fn read_header(file: &mut File) -> Result<Header, Error> {
        let mut header: Header = header::CLEAR_HEADER;
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(header.as_mut_slice())?;
        Ok(header)
    }
}
