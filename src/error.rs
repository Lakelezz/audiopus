use crate::ffi;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Error {
    /// A value failed to match a documented [`Application`].
    ///
    /// [`Application`]: ../enum.Application.html
    InvalidApplication,
    /// A value failed to match a documented [`Bandwidth`].
    ///
    /// [`Bandwidth`]: ../enum.Application.html
    InvalidBandwidth(i32),
    /// A value failed to match a documented [`Bitrate`],
    /// negative values are invalid.
    ///
    /// [`Bitrate`]: ../enum.Bitrate.html
    InvalidBitrate(i32),
    /// A value failed to match a documented [`Signal`].
    ///
    /// [`Signal`]: ../enum.Signal.html
    InvalidSignal(i32),
    /// Complexity was lower than 1 or higher than 10.
    InvalidComplexity(i32),
    /// A value failed to match a documented [`SampleRate`].
    ///
    /// [`SampleRate`]: ../enum.SampleRate.html
    InvalidSampleRate(i32),
    /// A value failed to match a documented [`Channel`].
    ///
    /// [`Channels`]: ../enum.Channels.html
    InvalidChannels(i32),
    /// An error returned from Opus containing an [`ErrorCode`] describing
    /// the cause.
    Opus(ErrorCode),
    /// Opus is not operating empty packets.
    EmptyPacket,
    /// Opus' maximum `Vec` or slice length of `std::i32::MAX` has been
    /// exceeded.
    SignalsTooLarge,
    /// Opus' maximum `Vec` or slice length of `std::i32::MAX` has been
    /// exceeded.
    PacketTooLarge,
    /// A `Vec` representing a mapping exceeded the expected value.
    MappingExpectedLen(usize),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::InvalidApplication => "Invalid Application",
            Error::InvalidBandwidth(_) => "Invalid Bandwidth",
            Error::InvalidSignal(_) => "Invalid Signal",
            Error::InvalidComplexity(_) => "Invalid Complexity",
            Error::InvalidSampleRate(_) => "Invalid Sample Rate",
            Error::InvalidChannels(_) => "Invalid Channels",
            Error::Opus(error_code) => error_code.description(),
            Error::EmptyPacket => "Passed packet contained no elements",
            Error::SignalsTooLarge => "Signals' length exceeded `std::i32::MAX`",
            Error::PacketTooLarge => "Packet's length exceeded `std::i32::MAX`",
            Error::InvalidBitrate(_) => "Invalid Bitrate",
            Error::MappingExpectedLen(_) => "Wrong channel length",
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                Error::InvalidApplication => self.description().to_string(),
                Error::InvalidBandwidth(bandwidth) => {
                    format!("{}: {}", self.description(), bandwidth)
                }
                Error::InvalidSignal(signal) => format!("{}: {}", self.description(), signal),
                Error::InvalidComplexity(complexity) => {
                    format!("{}: {}", self.description(), complexity)
                }
                Error::InvalidSampleRate(rate) => format!("{}: {}", self.description(), rate),
                Error::InvalidChannels(channels) => format!("{}: {}", self.description(), channels),
                Error::Opus(error_code) => format!("{}: {}", self.description(), &error_code),
                Error::EmptyPacket => self.description().to_string(),
                Error::SignalsTooLarge => self.description().to_string(),
                Error::PacketTooLarge => self.description().to_string(),
                Error::InvalidBitrate(rate) => format!("{}: {}", self.description(), rate),
                Error::MappingExpectedLen(len) => {
                    format!("{}, expected: {}", self.description(), len)
                }
            }
        )
    }
}

impl From<ErrorCode> for Error {
    fn from(error_code: ErrorCode) -> Error {
        Error::Opus(error_code)
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ErrorCode {
    BadArgument = ffi::OPUS_BAD_ARG,
    BufferTooSmall = ffi::OPUS_BUFFER_TOO_SMALL,
    InternalError = ffi::OPUS_INTERNAL_ERROR,
    InvalidPacket = ffi::OPUS_INVALID_PACKET,
    Unimplemented = ffi::OPUS_UNIMPLEMENTED,
    InvalidState = ffi::OPUS_INVALID_STATE,
    AllocFail = ffi::OPUS_ALLOC_FAIL,
    /// Occurs when Opus sends an error value that is not documented.
    /// `0` is unrelated to Opus and just a mere marker by this crate to
    /// differentiate between Opus' errors (all of them are negative).
    Unknown = 0,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.description())
    }
}

impl StdError for ErrorCode {
    fn description(&self) -> &str {
        match self {
            ErrorCode::BadArgument => "Passed argument violated Opus' specified requirements",
            ErrorCode::BufferTooSmall => "Passed buffer was too small",
            ErrorCode::InternalError => "Internal error inside Opus occured",
            ErrorCode::InvalidPacket => "Opus received a packet violating requirements",
            ErrorCode::Unimplemented => "Unimplemented code branch was attempted to be executed",
            ErrorCode::InvalidState => "Opus-type instance is in an invalid state",
            ErrorCode::AllocFail => "Opus was unable to allocate memory",
            ErrorCode::Unknown => {
                "Opus returned a non-negative error, this might be a Audiopus or Opus bug"
            }
        }
    }
}

impl From<i32> for ErrorCode {
    fn from(number: i32) -> ErrorCode {
        match number {
            ffi::OPUS_BAD_ARG => ErrorCode::BadArgument,
            ffi::OPUS_BUFFER_TOO_SMALL => ErrorCode::BufferTooSmall,
            ffi::OPUS_INTERNAL_ERROR => ErrorCode::InternalError,
            ffi::OPUS_INVALID_PACKET => ErrorCode::InvalidPacket,
            ffi::OPUS_UNIMPLEMENTED => ErrorCode::Unimplemented,
            ffi::OPUS_INVALID_STATE => ErrorCode::InvalidState,
            ffi::OPUS_ALLOC_FAIL => ErrorCode::AllocFail,
            _ => ErrorCode::Unknown,
        }
    }
}

/// Checks if the `ffi_return_value` is documented by Opus.
/// Returns `Error` if value is negative.
pub fn try_map_opus_error(ffi_return_value: i32) -> Result<i32> {
    match ffi_return_value {
        v if v < 0 => Err(Error::from(ErrorCode::from(v))),
        _ => Ok(ffi_return_value),
    }
}
