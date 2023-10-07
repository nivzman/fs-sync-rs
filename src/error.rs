
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IoError: {0}")]
    Io(#[from] std::io::Error),
    #[error("System time has gone backwards: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}
