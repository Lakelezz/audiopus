use super::GenericCtl;
use crate::{
    error::try_map_opus_error, ffi, packet::Packet, Channels, ErrorCode, MutSignals, Result,
    SampleRate, TryFrom, TryInto,
};

/// `Decoder` to decode.
#[derive(Debug)]
pub struct Decoder {
    pointer: *mut ffi::OpusDecoder,
    channels: Channels,
}

impl GenericCtl for Decoder {
    fn final_range(&self) -> Result<u32> {
        self.decoder_ctl_request(ffi::OPUS_GET_FINAL_RANGE_REQUEST)
            .map(|v| v as u32)
    }

    fn phase_inversion_disabled(&self) -> Result<bool> {
        self.decoder_ctl_request(ffi::OPUS_GET_PHASE_INVERSION_DISABLED_REQUEST)
            .map(|b| b == 1)
    }

    fn set_phase_inversion_disabled(&mut self, disabled: bool) -> Result<()> {
        let disable_phase_inversion = if disabled { 1 } else { 0 };
        self.set_decoder_ctl_request(
            ffi::OPUS_SET_PHASE_INVERSION_DISABLED_REQUEST,
            disable_phase_inversion,
        )
        .map(|_| ())
    }

    fn sample_rate(&self) -> Result<SampleRate> {
        self.decoder_ctl_request(ffi::OPUS_GET_SAMPLE_RATE_REQUEST)
            .and_then(SampleRate::try_from)
    }

    fn reset_state(&mut self) -> Result<()> {
        self.decoder_ctl_request(ffi::OPUS_RESET_STATE).map(|_| ())
    }
}

impl Decoder {
    /// Creates a new Opus decoder.
    pub fn new(sample_rate: SampleRate, channels: Channels) -> Result<Decoder> {
        let mut opus_code = 0;

        let pointer = unsafe {
            ffi::opus_decoder_create(sample_rate as i32, channels as i32, &mut opus_code)
        };

        if opus_code == ffi::OPUS_OK || !pointer.is_null() {
            return Ok(Decoder { pointer, channels });
        }

        Err(ErrorCode::from(opus_code))?
    }

    /// Decodes an Opus packet as `input` and writes decoded data into `output`.
    /// Passing `None` as `input` indicates a packet loss.
    ///
    /// **Errors**:
    /// If passed `input`'s length exceeds `std::i32::MAX`, [`PacketTooLarge`]
    /// will be returned.
    /// If passed `output`'s length exceeds `std::i32::MAX`, [`SignalsTooLarge`]
    /// will be returned.
    ///
    /// [`PacketTooLarge`]: ../error/enum.Error.html#variant.PacketTooLarge
    /// [`SignalsTooLarge`]: ../error/enum.Error.html#variant.SignalsTooLarge
    pub fn decode<'a, TP, TS>(&mut self, input: Option<TP>, output: TS, fec: bool) -> Result<usize>
    where
        TP: TryInto<Packet<'a>>,
        TS: TryInto<MutSignals<'a, i16>>,
    {
        let (input_pointer, input_len) = if let Some(value) = input {
            let value = value.try_into()?;

            (value.as_ptr(), value.i32_len())
        } else {
            (std::ptr::null(), 0)
        };

        let mut output = output.try_into()?;

        try_map_opus_error(unsafe {
            ffi::opus_decode(
                self.pointer,
                input_pointer,
                input_len,
                output.as_mut_ptr(),
                output.i32_len() / self.channels as i32,
                fec as i32,
            )
        })
        .map(|n| n as usize)
    }

    /// Decodes an Opus frame from floating point input.
    ///
    /// The `input` signal (interleaved if 2 channels) will be encoded into the
    /// `output` payload and on success, returns the length of the
    /// encoded packet.
    pub fn decode_float<'a, TP, TS>(&mut self, input: TP, output: TS, fec: bool) -> Result<usize>
    where
        TP: TryInto<Packet<'a>>,
        TS: TryInto<MutSignals<'a, f32>>,
    {
        let input = input.try_into()?;
        let mut output = output.try_into()?;

        try_map_opus_error(unsafe {
            ffi::opus_decode_float(
                self.pointer,
                input.as_ptr(),
                input.i32_len(),
                output.as_mut_ptr(),
                output.i32_len() / self.channels as i32,
                fec as i32,
            )
        })
        .map(|n| n as usize)
    }

    /// Issues a CTL `request` to Opus without argument used to
    /// request a value.
    /// If Opus returns a value smaller than 0, it indicates an error.
    fn decoder_ctl_request(&self, request: i32) -> Result<i32> {
        let mut value = 0;

        let ffi_result = unsafe { ffi::opus_decoder_ctl(self.pointer, request, &mut value) };

        try_map_opus_error(ffi_result)?;

        Ok(value)
    }

    /// Issues a CTL `request` to Opus accepting an additional argument used
    /// to set the `decoder`'s setting to `value`.
    /// If Opus returns a value smaller than 0, it indicates an error.
    fn set_decoder_ctl_request(&self, request: i32, value: i32) -> Result<()> {
        try_map_opus_error(unsafe { ffi::opus_decoder_ctl(self.pointer, request, value) })?;

        Ok(())
    }

    /// Gets the duration (in samples) of the last packet successfully decoded
    /// or concealed.
    pub fn last_packet_duration(&self) -> Result<u32> {
        self.decoder_ctl_request(ffi::OPUS_GET_LAST_PACKET_DURATION_REQUEST)
            .map(|v| v as u32)
    }

    /// Gets the pitch period at 48 kHz of the last decoded frame, if available.
    ///
    /// This can be used for any post-processing algorithm requiring the use of
    /// pitch, e.g. time stretching/shortening.
    /// If the last frame was not voiced, or if the pitch was not coded in the
    /// frame, then zero is returned.
    pub fn pitch(&self) -> Result<i32> {
        self.decoder_ctl_request(ffi::OPUS_GET_PITCH_REQUEST)
    }

    /// Gets the decoder's configured amount to scale PCM signal by
    /// in Q8 dB units.
    pub fn gain(&self) -> Result<i32> {
        self.decoder_ctl_request(ffi::OPUS_GET_GAIN_REQUEST)
    }

    /// Configures decoder gain adjustment.
    ///
    /// Scales the decoded output by a factor of `gain` specified in
    /// Q8 dB units.
    ///
    /// **Warning**:
    /// This has a maximum range of -32768 to 32767 inclusive, and returns
    /// [`BadArgument`] otherwise.
    /// The default is 0 indicating no adjustment.
    ///
    /// **Info**:
    /// This setting survives decoder reset.
    ///
    /// [`BadArgument`]: ../error/enum.ErrorCode.html#variant.BadArgument
    pub fn set_gain(&self, gain: i32) -> Result<()> {
        self.set_decoder_ctl_request(ffi::OPUS_SET_GAIN_REQUEST, gain)
    }

    /// Gets size of self's underlying Opus-decoder in bytes.
    pub fn size(&self) -> usize {
        unsafe { ffi::opus_decoder_get_size(self.channels as i32) as usize }
    }
}

/// Gets size of an Opus-decoder in bytes.
pub fn size(channels: Channels) -> usize {
    unsafe { ffi::opus_decoder_get_size(channels as i32) as usize }
}

impl Drop for Decoder {
    /// We have to ensure that the resource our wrapping Opus-struct is pointing
    /// to is deallocated properly.
    fn drop(&mut self) {
        unsafe { ffi::opus_decoder_destroy(self.pointer) }
    }
}

#[cfg(test)]
mod tests {
    use super::Decoder;
    use crate::{Channels, Error, ErrorCode, SampleRate};
    use matches::assert_matches;

    #[test]
    fn set_and_get_gain() {
        let decoder = Decoder::new(SampleRate::Hz48000, Channels::Stereo).unwrap();

        assert_matches!(decoder.gain(), Ok(0));

        assert_matches!(decoder.set_gain(10), Ok(()));

        assert_matches!(decoder.gain(), Ok(10));

        let lower_limit = -32768;
        let upper_limit = 32767;

        assert_matches!(decoder.set_gain(lower_limit), Ok(()));
        assert_matches!(
            decoder.set_gain(lower_limit - 1),
            Err(Error::Opus(ErrorCode::BadArgument))
        );

        assert_matches!(decoder.set_gain(upper_limit), Ok(()));
        assert_matches!(
            decoder.set_gain(upper_limit + 1),
            Err(Error::Opus(ErrorCode::BadArgument))
        );
    }
}
