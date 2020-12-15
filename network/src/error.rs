#[derive(Debug)]
pub enum Error {
    Cbor(serde_cbor::Error),
    Io(std::io::Error),
    DuplicateNetworkEntity,
}

impl From<serde_cbor::Error> for Error {
    fn from(error: serde_cbor::Error) -> Self {
        Error::Cbor(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}
