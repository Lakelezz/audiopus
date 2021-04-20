use crate::{
    error::try_map_opus_error, ffi, Bandwidth, Channels, Error, Result, SampleRate, TryFrom,
    TryInto,
};

fn packet_len_check(packet_buffer: &[u8]) -> Result<i32> {
    match packet_buffer {
        // non-empty guarantee:
        x if x.is_empty() => Err(Error::EmptyPacket),
        // limited size guarantee:
        _ if packet_buffer.len() > std::i32::MAX as usize => Err(Error::PacketTooLarge),
        _ => Ok(packet_buffer.len() as i32),
    }
}

/// A newtype around `&[u8]` to guarantee:
/// - Minimum one element: A packet cannot be empty.
/// - Limited size: A packet's length may not exceed `std::i32::MAX`.
#[derive(Debug)]
pub struct Packet<'a>(&'a [u8]);

impl<'a> Packet<'a> {
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// The underlying type is immutably borrowed and has been verified upon
    /// construction of `Packet`, thus we know casting `usize` will fit
    /// inside `i32`.
    pub fn i32_len(&self) -> i32 {
        self.0.len() as i32
    }
}

impl<'a> TryFrom<&'a Vec<u8>> for Packet<'a> {
    type Error = Error;

    fn try_from(value: &'a Vec<u8>) -> Result<Self> {
        value.as_slice().try_into()
    }
}

impl<'a> TryFrom<&'a [u8]> for Packet<'a> {
    type Error = Error;

    fn try_from(value: &'a [u8]) -> Result<Packet<'a>> {
        packet_len_check(value).map(|_| Self(value))
    }
}

/// A newtype around `&mut [u8]` to guarantee that accessing length on the
/// underlying buffer is checked each time.
#[derive(Debug)]
pub struct MutPacket<'a>(&'a mut [u8]);

impl<'a> MutPacket<'a> {
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    /// Checks if the underlying buffer meets requirements.
    pub fn i32_len(&self) -> Result<i32> {
        packet_len_check(&self.0)
    }
}

impl<'a> TryFrom<&'a mut Vec<u8>> for MutPacket<'a> {
    type Error = Error;

    fn try_from(value: &'a mut Vec<u8>) -> Result<Self> {
        value.as_mut_slice().try_into()
    }
}

impl<'a> TryFrom<&'a mut [u8]> for MutPacket<'a> {
    type Error = Error;

    fn try_from(value: &'a mut [u8]) -> Result<Self> {
        packet_len_check(value).map(move |_| Self(value))
    }
}

/// Gets bandwidth of an Opus `packet`.
///
/// **Errors**:
/// Empty `packet` will return `Error::EmptyPacket`.
pub fn bandwidth(packet: Packet<'_>) -> Result<Bandwidth> {
    unsafe { ffi::opus_packet_get_bandwidth(packet.as_ptr()) }.try_into()
}

/// Gets number of samples per frame of an Opus `packet`.
///
/// **Errors**:
/// Empty `packet` will return `Error::EmptyPacket`.
pub fn samples_per_frame(packet: Packet<'_>, sample_rate: SampleRate) -> Result<usize> {
    unsafe {
        Ok(ffi::opus_packet_get_samples_per_frame(packet.as_ptr(), sample_rate as i32) as usize)
    }
}

/// Gets number of samples in an Opus `packet`.
///
/// **Errors**:
/// Empty `packet` will return `Error::EmptyPacket`.
pub fn nb_samples(packet: Packet<'_>, sample_rate: SampleRate) -> Result<usize> {
    unsafe {
        try_map_opus_error(ffi::opus_packet_get_nb_samples(
            packet.as_ptr(),
            packet.i32_len(),
            sample_rate as i32,
        ))
        .map(|n| n as usize)
    }
}

/// Gets number of channels in an Opus `packet`.
///
/// **Errors**:
/// Empty `packet` will return `Error::EmptyPacket`.
pub fn nb_channels(packet: Packet<'_>) -> Result<Channels> {
    unsafe {
        Ok(Channels::try_from(ffi::opus_packet_get_nb_channels(
            packet.as_ptr(),
        ))?)
    }
}

/// Gets number of frames in an Opus `packet`.
///
/// **Errors**:
/// Empty `packet` will return [`Error::EmptyPacket`].
pub fn nb_frames(packet: Packet<'_>) -> Result<usize> {
    unsafe {
        try_map_opus_error(ffi::opus_packet_get_nb_frames(
            packet.as_ptr(),
            packet.i32_len(),
        ))
        .map(|n| n as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::bandwidth;
    use crate::{Bandwidth, Error, packet::Packet};
    use matches::assert_matches;

    #[test]
    /// We verify the `TryFrom`-impl for `Packet` by creating and then
    /// converting from `Vec`s that meet and violate the contract.
    fn packet_bandwidth() {
        use std::convert::TryFrom;

        let empty_packet = vec![];
        let empty_packet_bandwidth = Packet::try_from(&empty_packet);
        assert_matches!(empty_packet_bandwidth, Err(Error::EmptyPacket));

        let narrow_packet = vec![1, 2, 3];
        let narrow_packet_bandwidth = bandwidth(Packet::try_from(&narrow_packet).unwrap());
        assert_matches!(narrow_packet_bandwidth, Ok(Bandwidth::Narrowband));

        let mediumband_packet = vec![50];
        let mediumband_packet_bandwidth = bandwidth(Packet::try_from(&mediumband_packet).unwrap());
        assert_matches!(mediumband_packet_bandwidth, Ok(Bandwidth::Mediumband));

        let wideband_packet = vec![80];
        let wideband_packet_bandwidth = bandwidth(Packet::try_from(&wideband_packet).unwrap());
        assert_matches!(wideband_packet_bandwidth, Ok(Bandwidth::Wideband));

        let superwideband_packet = vec![200];
        let superwideband_bandwidth = bandwidth(Packet::try_from(&superwideband_packet).unwrap());
        assert_matches!(superwideband_bandwidth, Ok(Bandwidth::Superwideband));

        let fullband_packet = vec![255];
        let fullband_bandwidth = bandwidth(Packet::try_from(&fullband_packet).unwrap());
        assert_matches!(fullband_bandwidth, Ok(Bandwidth::Fullband));
    }
}
