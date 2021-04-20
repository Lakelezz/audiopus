//! Audiopus is a high level abstraction over the Opus library.
//!
//! This crate uses its own implementation of `TryInto`/`TryFrom` attempting to
//! convert to newtypes (`Packet`, `MutPacket`, and `MutSignals`) ensuring Opus'
//! restrictions will be kept in mind by checking these on construction.
//! Without these restrictions, crashes would occur because Opus does not know
//! any types larger than `i32` and does not expect empty packets.
//! Hence our own `TryInto`/`TryFrom` allows us to couple restriction-checks
//! with actual types and yet remaining stable Rust.
//! `Packet`, `MutPacket`, `MutSignals` implement conversions from
//! `&Vec[T]`/`&[T]`, they do not move or destroy the original type.
//!
//! A `Packet` references an underlying buffer of type `&[u8]`, it cannot be
//! empty and not longer than `std::i32::MAX`.
//!
//! Same goes for `MutPacket`, except the type mutably borrows the buffer thus
//! checks length before handing out the length-value as `Result`.
//!
//! `MutSignals` wraps around a generic buffer and represents Opus' output.
//! E.g. when encoding, Opus will fill the buffer with the encoded data.
//!
//! Audiopus aims to never panic or crash when interacting with Opus,
//! if either occurs, consider this a bug.
#![deny(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]
// TODO: Document all public items.
// #![deny(missing_docs)]

pub mod coder;
pub mod error;
pub mod packet;
pub mod repacketizer;
pub mod softclip;

use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
};

pub use crate::error::{Error, ErrorCode, Result};
pub use audiopus_sys as ffi;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Signal {
    Auto = ffi::OPUS_AUTO,
    Voice = ffi::OPUS_SIGNAL_VOICE,
    Music = ffi::OPUS_SIGNAL_MUSIC,
}

impl TryFrom<i32> for Signal {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        Ok(match value {
            ffi::OPUS_AUTO => Signal::Auto,
            ffi::OPUS_SIGNAL_VOICE => Signal::Voice,
            ffi::OPUS_SIGNAL_MUSIC => Signal::Music,
            _ => return Err(Error::InvalidSignal(value)),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Bitrate {
    /// Explicit bitrate choice (in bits/second).
    BitsPerSecond(i32),
    /// Maximum bitrate allowed (up to maximum number of bytes for the packet).
    Max,
    /// Default bitrate decided by the encoder (not recommended).
    Auto,
}

impl From<Bitrate> for i32 {
    fn from(bitrate: Bitrate) -> i32 {
        match bitrate {
            Bitrate::Auto => ffi::OPUS_AUTO,
            Bitrate::Max => ffi::OPUS_BITRATE_MAX,
            Bitrate::BitsPerSecond(bits) => bits,
        }
    }
}

impl TryFrom<i32> for Bitrate {
    type Error = Error;

    fn try_from(value: i32) -> Result<Bitrate> {
        Ok(match value {
            ffi::OPUS_AUTO => Bitrate::Auto,
            ffi::OPUS_BITRATE_MAX => Bitrate::Max,
            x if x.is_positive() => Bitrate::BitsPerSecond(x),
            _ => return Err(Error::InvalidBandwidth(value)),
        })
    }
}

/// Represents possible sample rates Opus can use.
/// Values represent Hertz.
#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SampleRate {
    Hz8000 = 8000,
    Hz12000 = 12000,
    Hz16000 = 16000,
    Hz24000 = 24000,
    Hz48000 = 48000,
}

impl TryFrom<i32> for SampleRate {
    type Error = Error;

    /// Fails if a number does not map a documented Opus sample rate.
    fn try_from(value: i32) -> Result<Self> {
        Ok(match value {
            8000 => SampleRate::Hz8000,
            12000 => SampleRate::Hz12000,
            16000 => SampleRate::Hz16000,
            24000 => SampleRate::Hz24000,
            48000 => SampleRate::Hz48000,
            _ => return Err(Error::InvalidSampleRate(value)),
        })
    }
}

/// Represents possible application-types for Opus.
#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Application {
    /// Best for most VoIP/videoconference applications where listening quality
    /// and intelligibility matter most.
    Voip = ffi::OPUS_APPLICATION_VOIP,
    /// Best for broadcast/high-fidelity application where the decoded audio
    /// should be as close as possible to the input.
    Audio = ffi::OPUS_APPLICATION_AUDIO,
    /// Only use when lowest-achievable latency is what matters most.
    LowDelay = ffi::OPUS_APPLICATION_RESTRICTED_LOWDELAY,
}

impl TryFrom<i32> for Application {
    type Error = Error;

    /// Fails if a value does not match Opus' specified application-value.
    fn try_from(value: i32) -> Result<Self> {
        Ok(match value {
            ffi::OPUS_APPLICATION_VOIP => Application::Voip,
            ffi::OPUS_APPLICATION_AUDIO => Application::Audio,
            ffi::OPUS_APPLICATION_RESTRICTED_LOWDELAY => Application::LowDelay,
            _ => return Err(Error::InvalidApplication),
        })
    }
}

/// Represents possible audio channels Opus can use.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Channels {
    /// Not supported when constructing encoders or decoders.
    Auto = ffi::OPUS_AUTO,
    Mono = 1,
    Stereo = 2,
}

impl Channels {
    pub fn is_mono(self) -> bool {
        if let Channels::Mono = self {
            return true;
        }

        false
    }

    pub fn is_stereo(self) -> bool {
        if let Channels::Stereo = self {
            return true;
        }

        false
    }
}

impl TryFrom<i32> for Channels {
    type Error = Error;

    // Fails if a value does not match Opus' specified channel-value.
    fn try_from(value: i32) -> Result<Channels> {
        Ok(match value {
            ffi::OPUS_AUTO => Channels::Auto,
            1 => Channels::Mono,
            2 => Channels::Stereo,
            _ => return Err(Error::InvalidChannels(value)),
        })
    }
}

impl From<Channels> for i32 {
    fn from(channels: Channels) -> i32 {
        channels as i32
    }
}

/// Represents possible bandwidths of an Opus-stream.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Bandwidth {
    /// Pick the bandwidth automatically.
    Auto = ffi::OPUS_AUTO,
    /// A 4kHz bandwidth.
    Narrowband = ffi::OPUS_BANDWIDTH_NARROWBAND,
    /// A 6kHz bandwidth.
    Mediumband = ffi::OPUS_BANDWIDTH_MEDIUMBAND,
    /// A 8kHz bandwidth.
    Wideband = ffi::OPUS_BANDWIDTH_WIDEBAND,
    /// A 12kHz bandwidth.
    Superwideband = ffi::OPUS_BANDWIDTH_SUPERWIDEBAND,
    /// A 20kHz bandwidth.
    Fullband = ffi::OPUS_BANDWIDTH_FULLBAND,
}

impl TryFrom<i32> for Bandwidth {
    type Error = Error;

    // Fails if a value does not match Opus' specified bandwidth-value.
    fn try_from(value: i32) -> Result<Self> {
        Ok(match value {
            ffi::OPUS_AUTO => Bandwidth::Auto,
            ffi::OPUS_BANDWIDTH_NARROWBAND => Bandwidth::Narrowband,
            ffi::OPUS_BANDWIDTH_MEDIUMBAND => Bandwidth::Mediumband,
            ffi::OPUS_BANDWIDTH_WIDEBAND => Bandwidth::Wideband,
            ffi::OPUS_BANDWIDTH_SUPERWIDEBAND => Bandwidth::Superwideband,
            ffi::OPUS_BANDWIDTH_FULLBAND => Bandwidth::Fullband,
            _ => return Err(Error::InvalidBandwidth(value)),
        })
    }
}

/// A newtype wrapping around a mutable buffer. They represent mutably borrowed
/// arguments that will be filled by Opus.
/// E.g. you pass this to an encode-method and Opus encodes data into the
/// underlying buffer.
///
/// **Info**:
/// This type is only verifying that Opus' requirement are not violated.
#[derive(Debug)]
pub struct MutSignals<'a, T>(&'a mut [T]);

impl<'a, T> TryFrom<&'a mut [T]> for MutSignals<'a, T> {
    type Error = Error;

    fn try_from(value: &'a mut [T]) -> Result<Self> {
        if value.len() > std::i32::MAX as usize {
            return Err(Error::SignalsTooLarge);
        }

        Ok(Self(value))
    }
}

impl<'a, T> TryFrom<&'a mut Vec<T>> for MutSignals<'a, T> {
    type Error = Error;

    fn try_from(value: &'a mut Vec<T>) -> Result<Self> {
        value.as_mut_slice().try_into()
    }
}

impl<'a, T> MutSignals<'a, T> {
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr()
    }

    /// Due to checking the length during construction of this newtype wrapping
    /// around a immutably borrowed buffer, we can safely cast `usize` to `i32`
    /// without worrying about `usize` being too large for `i32`.
    pub fn i32_len(&self) -> i32 {
        self.0.len() as i32
    }
}

/// Gets the libopus version string.
///
/// Applications may look for the substring "-fixed" in the version string to
/// determine whether they have a fixed-point or floating-point build at runtime.
pub fn version() -> &'static str {
    // The pointer given from the `opus_get_version_string` function will be valid
    // therefore we can create a `CStr` from this pointer.
    unsafe { CStr::from_ptr(ffi::opus_get_version_string()) }
        .to_str()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::{ffi, version, Application, Error, Signal, TryFrom};
    use matches::assert_matches;

    #[test]
    fn try_get_version() {
        // We can't actually check the contents of the string, as it will change when the version
        // changes. By just calling the function we can ensure that the CStr conversion succeeds.
        version();
    }

    #[test]
    fn signal_try_from() {
        assert_matches!(Signal::try_from(ffi::OPUS_SIGNAL_MUSIC), Ok(Signal::Music));
        assert_matches!(Signal::try_from(ffi::OPUS_SIGNAL_VOICE), Ok(Signal::Voice));
        assert_matches!(Signal::try_from(ffi::OPUS_AUTO), Ok(Signal::Auto));
        assert_matches!(Signal::try_from(0), Err(Error::InvalidSignal(0)));
    }

    #[test]
    fn application_try_from() {
        assert_matches!(
            Application::try_from(ffi::OPUS_APPLICATION_AUDIO),
            Ok(Application::Audio)
        );
        assert_matches!(
            Application::try_from(ffi::OPUS_APPLICATION_VOIP),
            Ok(Application::Voip)
        );
        assert_matches!(
            Application::try_from(ffi::OPUS_APPLICATION_RESTRICTED_LOWDELAY),
            Ok(Application::LowDelay)
        );
        assert_matches!(Application::try_from(11), Err(Error::InvalidApplication));
    }
}
