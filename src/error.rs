use std::error::Error as StdError;
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

// NOTE: these are not constructed directly outside of FFI
#[allow(unused)]
#[repr(C)]
pub(crate) enum TVResult {
    Ok = 0,
    Argument = -1,
    ArgumentNull = -2,
    KeyFromDer = -3,
    Context = -4,
    BufferTooSmall = -5,
    Base58Decode = -6,
    SocketBind = -7,
    EngineStart = -8,
    MessageReceive = -9,
    TransactionSendClosed = -10,
    TransactionDataTooLarge = -11,
}

impl TVResult {
    /// Converts the C TVResult into a Result type used in Rust.
    #[inline(always)]
    pub(crate) fn ok(self) -> Result<()> {
        self.ok_with(())
    }

    /// Converts the C TVResult into a Result type used in Rust.
    pub(crate) fn ok_with<T>(self, success: T) -> Result<T> {
        match self {
            Self::Ok => Ok(success),
            Self::Argument => Err(Error::Argument),
            Self::ArgumentNull => Err(Error::ArgumentNull),
            Self::KeyFromDer => Err(Error::KeyFromDer),
            Self::Context => Err(Error::Context),
            Self::BufferTooSmall => Err(Error::BufferTooSmall),
            Self::Base58Decode => Err(Error::Base58Decode),
            Self::SocketBind => Err(Error::SocketBind),
            Self::EngineStart => Err(Error::EngineStart),
            Self::MessageReceive => Err(Error::MessageReceive),
            Self::TransactionSendClosed => Err(Error::TransactionSendClosed),
            Self::TransactionDataTooLarge => Err(Error::TransactionDataTooLarge),
        }
    }

    /// Asserts that the result is OK, panicking otherwise.
    #[inline(always)]
    pub(crate) fn assert_ok(self) {
        self.ok().unwrap();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// An argument provided to a function was invalid.
    Argument,

    /// A required argument provided to a function was NULL.
    ArgumentNull,

    /// Failed to parse a key from DER format.
    KeyFromDer,

    /// Failed to initialize a new Context.
    Context,

    /// A provided buffer was too small to hold the required data.
    BufferTooSmall,

    /// Failed to decode a Base58 string.
    Base58Decode,

    /// Failed to bind a socket.
    SocketBind,

    /// Failed to start the engine.
    EngineStart,

    /// General failure to receive a message. Likely the stream has been closed.
    MessageReceive,

    /// Attempt to send a transaction on a shutdown engine.
    TransactionSendClosed,

    /// The transaction data exceeds the maximum allowed size (512 MiB).
    TransactionDataTooLarge,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Self::Argument => "An argument provided to a function was invalid.",
            Self::ArgumentNull => "A required argument provided to a function was NULL.",
            Self::KeyFromDer => "Failed to parse a key from DER format.",
            Self::Context => "Failed to initialize a new Context.",
            Self::BufferTooSmall => "A provided buffer was too small to hold the required data.",
            Self::Base58Decode => "Failed to decode a Base58 string.",
            Self::SocketBind => "Failed to bind a socket.",
            Self::EngineStart => "Failed to start the engine.",
            Self::MessageReceive => {
                "General failure to receive a message. Likely the stream has been closed."
            }
            Self::TransactionSendClosed => "Attempt to send a transaction on a shutdown engine.",
            Self::TransactionDataTooLarge => {
                "The transaction data exceeds the maximum allowed size (512 MiB)."
            }
        }
    }
}

impl Display for Error {
    #[allow(deprecated)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
