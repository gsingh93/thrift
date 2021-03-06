#![feature(core)]
extern crate podio;

pub use protocol::Protocol;
pub use transport::Transport;

pub mod protocol;
pub mod transport;

#[derive(Eq, PartialEq, Debug)]
pub enum ThriftErr {
    TransportError(std::io::Error),
    UnknownProtocol,
    InvalidData,
    NegativeSize,
    SizeLimit,
    BadVersion,
    NotImplemented,
    DepthLimit,
    InvalidUtf8(std::str::Utf8Error),
    Exception,
    ProtocolError,
}

impl std::error::FromError<std::io::Error> for ThriftErr {
	fn from_error(err: std::io::Error) -> ThriftErr {
		ThriftErr::TransportError(err)
	}
}

pub type TResult<T> = Result<T, ThriftErr>;
