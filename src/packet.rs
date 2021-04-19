use crate::{error::try_map_opus_error, ffi, Bandwidth, Channels, Error, Result, SampleRate, TryFrom, TryInto};

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

impl<'a> TryInto<Packet<'a>> for &'a Vec<u8> {
    type Error = Error;

    fn try_into(self) -> Result<Packet<'a>> {
        self[..].try_into()
    }
}

impl<'a> TryInto<Packet<'a>> for &'a [u8] {
    type Error = Error;

    fn try_into(self) -> Result<Packet<'a>> {
        packet_len_check(self).map(|_| Packet(self))
    }
}

impl<'a> TryInto<Packet<'a>> for &'a Packet<'a> {
    type Error = Error;

    fn try_into(self) -> Result<Packet<'a>> {
        Ok(Packet(self.0))
    }
}

/// A newtype around `&mut [u8]` to guarantee that accessing length on the
/// underlying buffer is checked each time.
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

impl<'a> TryInto<MutPacket<'a>> for &'a mut Vec<u8> {
    type Error = Error;

    fn try_into(self) -> Result<MutPacket<'a>> {
        self.as_mut_slice().try_into()
    }
}

impl<'a> TryInto<MutPacket<'a>> for &'a mut [u8] {
    type Error = Error;

    fn try_into(self) -> Result<MutPacket<'a>> {
        packet_len_check(self).map(move |_| MutPacket(self))
    }
}

/// Gets bandwidth of an Opus `packet`.
///
/// **Warning**:
/// Empty `packet` will return `Error::EmptyPacket`.
pub fn bandwidth<'a, I>(packet: I) -> Result<Bandwidth>
where
    I: TryInto<Packet<'a>, Error = Error>,
{
    let packet = packet.try_into()?;

    unsafe { Bandwidth::try_from(ffi::opus_packet_get_bandwidth(packet.as_ptr())) }
}

/// Gets number of samples per frame of an Opus `packet`.
///
/// **Warning**:
/// Empty `packet` will return `Error::EmptyPacket`.
pub fn samples_per_frame<'a, I>(packet: I, sample_rate: SampleRate) -> Result<usize>
where
    I: TryInto<Packet<'a>, Error = Error>,
{
    let packet = packet.try_into()?;

    unsafe {
        Ok(ffi::opus_packet_get_samples_per_frame(packet.as_ptr(), sample_rate as i32) as usize)
    }
}

/// Gets number of samples in an Opus `packet`.
///
/// **Warning**:
/// Empty `packet` will return `Error::EmptyPacket`.
pub fn nb_samples<'a, I>(packet: I, sample_rate: SampleRate) -> Result<usize>
where
    I: TryInto<Packet<'a>, Error = Error>,
{
    let packet = packet.try_into()?;

    unsafe {
        try_map_opus_error(ffi::opus_packet_get_nb_samples(packet.as_ptr(), packet.i32_len(), sample_rate as i32))
            .map(|n| n as usize)
    }
}

/// Gets number of channels in an Opus `packet`.
///
/// **Warning**:
/// Empty `packet` will return `Error::EmptyPacket`.
pub fn nb_channels<'a, I>(packet: I) -> Result<Channels>
where
    I: TryInto<Packet<'a>, Error = Error>,
{
    let packet = packet.try_into()?;

    unsafe {
        Ok(Channels::try_from(ffi::opus_packet_get_nb_channels(
            packet.as_ptr(),
        ))?)
    }
}

/// Gets number of frames in an Opus `packet`.
///
/// **Warning**:
/// Empty `packet` will return `Error::EmptyPacket`.
pub fn nb_frames<'a, I>(packet: I) -> Result<usize>
where
    I: TryInto<Packet<'a>, Error = Error>,
{
    let packet = packet.try_into()?;

    unsafe {
        try_map_opus_error(ffi::opus_packet_get_nb_frames(packet.as_ptr(), packet.i32_len()))
            .map(|n| n as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::bandwidth;
    use crate::{Bandwidth, Error};
    use matches::assert_matches;

    #[test]
    fn packet_bandwidth() {
        let empty_packet = vec![];
        let empty_packet_bandwidth = bandwidth(&empty_packet);
        assert_matches!(empty_packet_bandwidth, Err(Error::EmptyPacket));

        let narrow_packet = vec![1, 2, 3];
        let narrow_packet_bandwidth = bandwidth(&narrow_packet);
        assert_matches!(narrow_packet_bandwidth, Ok(Bandwidth::Narrowband));

        let mediumband_packet = vec![50];
        let mediumband_packet_bandwidth = bandwidth(&mediumband_packet);
        assert_matches!(mediumband_packet_bandwidth, Ok(Bandwidth::Mediumband));

        let wideband_packet = vec![80];
        let wideband_packet_bandwidth = bandwidth(&wideband_packet);
        assert_matches!(wideband_packet_bandwidth, Ok(Bandwidth::Wideband));

        let superwideband_packet = vec![200];
        let superwideband_bandwidth = bandwidth(&superwideband_packet);
        assert_matches!(superwideband_bandwidth, Ok(Bandwidth::Superwideband));

        let fullband_packet = vec![255];
        let fullband_bandwidth = bandwidth(&fullband_packet);
        assert_matches!(fullband_bandwidth, Ok(Bandwidth::Fullband));
    }
}
