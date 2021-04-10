use super::GenericCtl;
use crate::{
    error::try_map_opus_error, ffi, Application, Bandwidth, Bitrate, Channels, ErrorCode, Result,
    SampleRate, Signal, TryFrom,
};

/// `Encoder` calls to Opus and offers method to encode and issue
/// requests to Opus.
#[derive(Debug)]
pub struct Encoder {
    pointer: *mut ffi::OpusEncoder,
    channels: Channels,
}

/// The Opus encoder can be sent between threads unless the Opus library
/// has been compiled with `NONTHREADSAFE_PSEUDOSTACK` to disallow encoding in
/// parallel.
unsafe impl Send for Encoder {}

impl GenericCtl for Encoder {
    /// Gets the final state of the codec's entropy coder.
    ///
    /// This is used for testing purposes. The encoder state should
    /// be identical after coding a payload, assuming no data corruption or
    /// software bugs.
    fn final_range(&self) -> Result<u32> {
        self.encoder_ctl_request(ffi::OPUS_GET_FINAL_RANGE_REQUEST)
            .map(|v| v as u32)
    }

    /// Gets the encoder's configured phase inversion status.
    fn phase_inversion_disabled(&self) -> Result<bool> {
        self.encoder_ctl_request(ffi::OPUS_SET_PHASE_INVERSION_DISABLED_REQUEST)
            .map(|b| b == 1)
    }

    /// If set to `true`, disables the use of phase inversion for intensity
    /// stereo, improving the quality of mono downmixes, but slightly reducing
    /// normal stereo quality.
    ///
    /// Disabling phase inversion in the decoder does not comply with RFC 6716,
    /// although it does not cause any interoperability issue and is expected
    /// to become part of the Opus standard once RFC 6716 is updated by
    /// draft-ietf-codec-opus-update.
    fn set_phase_inversion_disabled(&mut self, disabled: bool) -> Result<()> {
        let disable_phase_inversion = if disabled { 1 } else { 0 };
        self.set_encoder_ctl_request(
            ffi::OPUS_SET_PHASE_INVERSION_DISABLED_REQUEST,
            disable_phase_inversion,
        )
        .map(|_| ())
    }

    /// Gets the sampling rate the encoder or decoder was initialized with.
    ///
    /// This simply returns the Fs value passed to [`Encoder::new`].
    ///
    /// [`Encoder::new`]: struct.Encoder.html#method.new
    fn sample_rate(&self) -> Result<SampleRate> {
        self.encoder_ctl_request(ffi::OPUS_GET_SAMPLE_RATE_REQUEST)
            .and_then(SampleRate::try_from)
    }

    /// Resets the codec state to be equivalent to a freshly initialized state.
    ///
    /// This should be called when switching streams in order to prevent the
    /// back to back decoding from giving different results from one at a
    /// time decoding.
    fn reset_state(&mut self) -> Result<()> {
        self.encoder_ctl_request(ffi::OPUS_RESET_STATE).map(|_| ())
    }
}

impl Encoder {
    /// Creates a new Opus encoder.
    ///
    /// **Warning**:
    /// If `channels` is set to [`Channels::Auto`] the function will
    /// return [`BadArgument`].
    ///
    /// [`Channels::Auto`]: ../enum.Channels.html#variant.Auto
    /// [`BadArgument`]: ../error/enum.ErrorCode.html#variant.BadArgument
    pub fn new(sample_rate: SampleRate, channels: Channels, mode: Application) -> Result<Encoder> {
        let mut opus_code = 0;

        let pointer = unsafe {
            ffi::opus_encoder_create(
                sample_rate as i32,
                channels as i32,
                mode as i32,
                &mut opus_code,
            )
        };

        if opus_code == ffi::OPUS_OK || !pointer.is_null() {
            return Ok(Encoder { pointer, channels });
        }

        Err(ErrorCode::from(opus_code))?
    }

    /// Issues a CTL get-`request` to Opus.
    /// If Opus returns a negative value it indicates an error.
    ///
    /// **Info**:
    /// As [`Encoder`]'s methods cover all possible CTLs, it is recommended
    /// to use them instead.
    ///
    /// [`Encoder`]: struct.Encoder.html
    pub fn encoder_ctl_request(&self, request: i32) -> Result<i32> {
        let mut value = 0;

        let ffi_result = unsafe { ffi::opus_encoder_ctl(self.pointer, request, &mut value) };
        try_map_opus_error(ffi_result)?;

        Ok(value)
    }

    /// Issues a CTL set-`request` to Opus and sets the `Encoder`'s setting to
    /// `value` based on sent `request`.
    /// If Opus returns a negative value it indicates an error.
    ///
    /// **Info**:
    /// As [`Encoder`]'s methods cover all possible CTLs, it is recommended
    /// to use them instead.
    ///
    /// [`Encoder`]: struct.Encoder.html
    pub fn set_encoder_ctl_request(&mut self, request: i32, value: i32) -> Result<()> {
        try_map_opus_error(unsafe { ffi::opus_encoder_ctl(self.pointer, request, value) })?;

        Ok(())
    }

    /// Encodes an Opus frame.
    ///
    /// The `input` signal (interleaved if 2 channels) will be encoded into the
    /// `output` payload and on success returns the length of the
    /// encoded packet.
    pub fn encode(&self, input: &[i16], output: &mut [u8]) -> Result<usize> {
        try_map_opus_error(unsafe {
            ffi::opus_encode(
                self.pointer,
                input.as_ptr(),
                input.len() as i32 / self.channels as i32,
                output.as_mut_ptr(),
                output.len() as i32,
            )
        })
        .map(|n| n as usize)
    }

    /// Encodes an Opus frame from floating point input.
    ///
    /// The `input` signal (interleaved if 2 channels) will be encoded into the
    /// `output` payload and on success, returns the length of the
    /// encoded packet.
    pub fn encode_float(&self, input: &[f32], output: &mut [u8]) -> Result<usize> {
        try_map_opus_error(unsafe {
            ffi::opus_encode_float(
                self.pointer,
                input.as_ptr(),
                input.len() as i32 / self.channels as i32,
                output.as_mut_ptr(),
                output.len() as i32,
            )
        })
        .map(|n| n as usize)
    }

    /// Gets the encoder's complexity configuration.
    pub fn complexity(&self) -> Result<u8> {
        self.encoder_ctl_request(ffi::OPUS_GET_COMPLEXITY_REQUEST)
            .map(|v| v as u8)
    }

    /// Configures the encoder's computational complexity.
    ///
    /// **Warning**:
    /// If `complexity` exceeds 10, [`BadArgument`] will be returned.
    ///
    /// [`BadArgument`]: ../error/enum.ErrorCode.html#variant.BadArgument.html
    pub fn set_complexity(&mut self, complexity: u8) -> Result<()> {
        self.set_encoder_ctl_request(ffi::OPUS_SET_COMPLEXITY_REQUEST, i32::from(complexity))
    }

    /// Gets the encoder's configured application.
    pub fn application(&self) -> Result<Application> {
        self.encoder_ctl_request(ffi::OPUS_GET_APPLICATION_REQUEST)
            .and_then(Application::try_from)
    }

    /// Configures the encoder's intended application.
    ///
    /// The initial value is a mandatory argument in the [`new`]-function.
    ///
    /// [`new`]: struct.Encoder.html#method.new
    pub fn set_application(&mut self, application: Application) -> Result<()> {
        self.set_encoder_ctl_request(ffi::OPUS_SET_APPLICATION_REQUEST, application as i32)
            .map(|_| ())
    }

    /// Configures the bitrate in the encoder.
    ///
    /// Rates from 500 to 512000 bits per second are meaningful,
    /// as well as the special values [`Bitrate::Auto`] and [`Bitrate::Max`].
    /// [`Bitrate::Max`] can be used to cause the codec to use
    /// as much rate as it can, which is useful for controlling the rate by
    /// adjusting the output buffer size.
    pub fn set_bitrate(&mut self, bitrate: Bitrate) -> Result<()> {
        self.set_encoder_ctl_request(ffi::OPUS_SET_BITRATE_REQUEST, bitrate.into())?;

        Ok(())
    }

    /// Gets the encoder's configured bandpass.
    pub fn bitrate(&self) -> Result<Bitrate> {
        self.encoder_ctl_request(ffi::OPUS_GET_BITRATE_REQUEST)
            .and_then(Bitrate::try_from)
    }

    /// Enables variable bitrate (VBR) in the encoder.
    ///
    /// The configured bitrate may not be met exactly because frames must be an
    /// integer number of bytes in length.
    ///
    /// **Warning**:
    /// Only the MDCT mode of Opus currently heeds the constraint.
    /// Speech mode ignores it completely,
    /// hybrid mode may fail to obey it if the LPC layer uses more bitrate
    /// than the constraint would have permitted.
    pub fn enable_vbr_constraint(&mut self) -> Result<()> {
        self.set_vbr_constraint(true)
    }

    /// Disables variable bitrate (VBR) in the encoder.
    ///
    /// The configured bitrate may not be met exactly because frames must be an
    /// integer number of bytes in length.
    ///
    /// **Warning**:
    /// Only the MDCT mode of Opus currently heeds the constraint.
    /// Speech mode ignores it completely,
    /// hybrid mode may fail to obey it if the LPC layer uses more bitrate
    /// than the constraint would have permitted.
    pub fn disable_vbr_constraint(&mut self) -> Result<()> {
        self.set_vbr_constraint(false)
    }

    /// Sets variable bitrate (VBR) in the encoder.
    ///
    /// The configured bitrate may not be met exactly because frames must be an
    /// integer number of bytes in length.
    ///
    /// **Warning**:
    /// Only the MDCT mode of Opus currently heeds the constraint.
    /// Speech mode ignores it completely,
    /// hybrid mode may fail to obey it if the LPC layer uses more bitrate
    /// than the constraint would have permitted.
    pub fn set_vbr_constraint(&mut self, enable: bool) -> Result<()> {
        let if_vbr_shall_be_enabled = if enable { 1 } else { 0 };

        self.set_encoder_ctl_request(
            ffi::OPUS_SET_VBR_CONSTRAINT_REQUEST,
            if_vbr_shall_be_enabled,
        )
        .map(|_| ())
    }

    /// Determine if constrained VBR is enabled in the encoder.
    pub fn vbr_constraint(&self) -> Result<bool> {
        self.encoder_ctl_request(ffi::OPUS_GET_VBR_CONSTRAINT_REQUEST)
            .map(|b| b == 1)
    }

    /// Enables variable bitrate (VBR) in the encoder.
    ///
    /// The configured bitrate may not be met exactly because frames must be an
    /// integer number of bytes in length.
    pub fn enable_vbr(&mut self) -> Result<()> {
        self.set_vbr(true)
    }

    /// Disables variable bitrate (VBR) in the encoder.
    ///
    /// The configured bitrate may not be met exactly because frames must be an
    /// integer number of bytes in length.
    pub fn disable_vbr(&mut self) -> Result<()> {
        self.set_vbr(false)
    }

    /// Sets variable bitrate (VBR) in the encoder.
    ///
    /// The configured bitrate may not be met exactly because frames must be an
    /// integer number of bytes in length.
    pub fn set_vbr(&mut self, enable: bool) -> Result<()> {
        let if_vbr_shall_be_enabled = if enable { 1 } else { 0 };

        self.set_encoder_ctl_request(ffi::OPUS_SET_VBR_REQUEST, if_vbr_shall_be_enabled)
            .map(|_| ())
    }

    /// Determine if variable bitrate (VBR) is enabled in the encoder.
    pub fn vbr(&self) -> Result<bool> {
        self.encoder_ctl_request(ffi::OPUS_GET_VBR_REQUEST)
            .map(|b| b == 1)
    }

    /// Configures the encoder's use of inband forward error correction (FEC).
    pub fn set_inband_fec(&mut self, enable: bool) -> Result<()> {
        let if_inband_fec_shall_be_enabled = if enable { 1 } else { 0 };

        self.set_encoder_ctl_request(
            ffi::OPUS_SET_INBAND_FEC_REQUEST,
            if_inband_fec_shall_be_enabled,
        )
        .map(|_| ())
    }

    /// Enables the encoder's use of inband forward error correction (FEC).
    pub fn enable_inband_fec(&mut self) -> Result<()> {
        self.set_inband_fec(true)
    }

    /// Disables the encoder's use of inband forward error correction (FEC).
    pub fn disable_inband_fec(&mut self) -> Result<()> {
        self.set_inband_fec(false)
    }

    /// Gets encoder's configured use of inband forward error correction.
    pub fn inband_fec(&self) -> Result<bool> {
        self.encoder_ctl_request(ffi::OPUS_GET_INBAND_FEC_REQUEST)
            .map(|n| n == 1)
    }

    /// Gets the encoder's configured packet loss percentage.
    pub fn packet_loss_perc(&self) -> Result<u8> {
        self.encoder_ctl_request(ffi::OPUS_GET_PACKET_LOSS_PERC_REQUEST)
            .map(|n| n as u8)
    }

    /// Higher values trigger progressively more loss resistant behavior in the
    /// encoder at the expense of quality at a given bitrate in the absence of
    /// packet loss, but greater quality under loss.
    ///
    /// Configures the encoder's expected packet loss percentage.
    pub fn set_packet_loss_perc(&mut self, percentage: u8) -> Result<()> {
        self.set_encoder_ctl_request(
            ffi::OPUS_SET_PACKET_LOSS_PERC_REQUEST,
            i32::from(percentage),
        )
        .map(|_| ())
    }

    /// Gets the total samples of delay added by the entire codec.
    ///
    /// This can be queried by the encoder and then the provided number of
    /// samples can be skipped on from the start of the decoder's output to
    /// provide time aligned input and output. From the perspective of a
    /// decoding application the real data begins this many samples late.
    ///
    /// The decoder contribution to this delay is identical for all decoders,
    /// but the encoder portion of the delay may vary from implementation to
    /// implementation, version to version, or even depend on the encoder's
    /// initial configuration.
    /// Applications needing delay compensation should call this method
    /// rather than hard-coding a value.
    pub fn lookahead(&self) -> Result<u32> {
        self.encoder_ctl_request(ffi::OPUS_GET_LOOKAHEAD_REQUEST)
            .map(|n| n as u32)
    }

    /// Configures mono/stereo forcing in the encoder.
    ///
    /// This can force the encoder to produce packets encoded as either
    /// mono or stereo, regardless of the format of the input audio.
    /// This is useful when the caller knows that the input signal is
    /// currently a mono source embedded in a stereo stream.
    pub fn set_force_channels(&mut self, channels: Channels) -> Result<()> {
        self.set_encoder_ctl_request(ffi::OPUS_SET_FORCE_CHANNELS_REQUEST, channels as i32)
            .map(|_| ())
    }

    /// Gets the encoder's forced channel configuration.
    pub fn force_channels(&self) -> Result<Channels> {
        self.encoder_ctl_request(ffi::OPUS_GET_FORCE_CHANNELS_REQUEST)
            .and_then(Channels::try_from)
    }

    /// Gets the encoder's configured maximum allowed bandpass.
    pub fn max_bandwidth(&self) -> Result<Bandwidth> {
        self.encoder_ctl_request(ffi::OPUS_GET_MAX_BANDWIDTH_REQUEST)
            .and_then(Bandwidth::try_from)
    }

    /// Configures the maximum bandpass that the encoder will select automatically.
    ///
    /// Applications should normally use this instead of [`set_bandwidth`]
    /// (leaving that set to the default, [`Bandwidth::Auto`]).
    ///
    /// This allows the application to set an upper bound based on the type of
    /// input it is providing, but still gives the encoder the freedom to reduce
    /// the bandpass when the bitrate becomes too low,
    /// for better overall quality.
    ///
    /// **Warning**:
    /// [`Bandwidth::Auto`] will return [`BadArgument`] as it is not
    /// accepted by Opus as `bandwidth` value.
    ///
    /// [`set_bandwidth`]: struct.Encoder.html#method.set_bandwidth.html
    /// [`Bandwidth::Auto`]: ../enum.Bandwidth.html#variant.Auto
    /// [`BadArgument`]: ../error/enum.ErrorCode.html#variant.BadArgument.html
    pub fn set_max_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<()> {
        self.set_encoder_ctl_request(ffi::OPUS_SET_MAX_BANDWIDTH_REQUEST, bandwidth as i32)
    }

    /// Gets the encoder's configured prediction status.
    pub fn prediction_disabled(&self) -> Result<bool> {
        self.encoder_ctl_request(ffi::OPUS_GET_PREDICTION_DISABLED_REQUEST)
            .map(|n| n == 1)
    }

    /// If set `prediction_disabled` to `true`, disables almost all use of
    /// prediction, making frames almost completely independent.
    ///
    /// This reduces quality.
    pub fn set_prediction_disabled(&mut self, prediction_disabled: bool) -> Result<()> {
        let prediction_disabled = if prediction_disabled { 1 } else { 0 };

        self.set_encoder_ctl_request(
            ffi::OPUS_SET_PREDICTION_DISABLED_REQUEST,
            prediction_disabled,
        )
        .map(|_| ())
    }

    /// Gets the encoder's configured signal type.
    pub fn signal(&self) -> Result<Signal> {
        self.encoder_ctl_request(ffi::OPUS_GET_SIGNAL_REQUEST)
            .and_then(Signal::try_from)
    }

    /// Configures the type of signal being encoded.
    ///
    /// This is a hint which helps the encoder's mode selection.
    pub fn set_signal(&mut self, signal: Signal) -> Result<()> {
        self.set_encoder_ctl_request(ffi::OPUS_SET_SIGNAL_REQUEST, signal as i32)
            .map(|_| ())
    }

    /// Gets the encoder's configured bandpass.
    pub fn bandwidth(&self) -> Result<Bandwidth> {
        self.encoder_ctl_request(ffi::OPUS_GET_BANDWIDTH_REQUEST)
            .and_then(Bandwidth::try_from)
    }

    /// Sets the encoder's bandpass to a specific value.
    ///
    /// This prevents the encoder from automatically selecting the bandpass
    /// based on the available bitrate.
    /// If an application knows the bandpass of the input audio it is providing,
    /// it should normally use [`set_max_bandwidth`] instead,
    /// which still gives the encoder the freedom to reduce the bandpass when
    /// the bitrate becomes too low, for better overall quality.
    ///
    /// [`set_max_bandwidth`]: struct.Encoder.html#method.set_max_bandwidth
    pub fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<()> {
        self.set_encoder_ctl_request(ffi::OPUS_SET_BANDWIDTH_REQUEST, bandwidth as i32)
    }

    /// Gets encoder's configured use of discontinuous transmission.
    pub fn dtx(&self) -> Result<bool> {
        self.encoder_ctl_request(ffi::OPUS_GET_DTX_REQUEST)
            .map(|n| n == 1)
    }

    /// Configures the encoder's use of discontinuous transmission (DTX).
    pub fn set_dtx(&mut self, dtx: bool) -> Result<()> {
        let dtx_shall_be_enabled = if dtx {1} else {0};

        self.set_encoder_ctl_request(ffi::OPUS_SET_DTX_REQUEST, dtx_shall_be_enabled)
            .map(|_| ())
    }

    /// Enables the encoder's use of discontinuous transmission (DTX).
    pub fn enable_dtx(&mut self) -> Result<()> {
        self.set_dtx(true)
    }

    /// Disables the encoder's use of discontinuous transmission (DTX).
    pub fn disable_dtx(&mut self) -> Result<()> {
        self.set_dtx(false)
    }

    /// Gets the encoder's configured signal depth.
    pub fn lsb_depth(&self) -> Result<u8> {
        self.encoder_ctl_request(ffi::OPUS_GET_LSB_DEPTH_REQUEST)
            .map(|n| n as u8)
    }

    /// Configures the depth of signal being encoded.
    ///
    /// This is a hint which helps the encoder identify silence and near-silence.
    /// It represents the number of significant bits of linear intensity below
    /// which the signal contains ignorable quantisation or other noise.
    ///
    /// For example, a depth of 14 would be an appropriate setting for G.711
    /// u-law input. A depth of 16 would be appropriate for 16-bit linear pcm
    /// input with `encode_float()`.
    ///
    /// When using `encode()` instead of `encode_float()`, or when libopus is
    /// compiled for fixed-point, the encoder uses the minimum of the value set
    /// here and the value 16.
    pub fn set_lsb_depth(&mut self, lsb_depth: u8) -> Result<()> {
        self.set_encoder_ctl_request(ffi::OPUS_SET_LSB_DEPTH_REQUEST, i32::from(lsb_depth))
            .map(|_| ())
    }
}

impl Drop for Encoder {
    /// We have to ensure that the resource our wrapping Opus-struct is pointing
    /// to is deallocated properly.
    fn drop(&mut self) {
        unsafe { ffi::opus_encoder_destroy(self.pointer) }
    }
}

#[cfg(test)]
mod tests {
    use super::Encoder;
    use crate::{Application, Bandwidth, Bitrate, Channels, Error, ErrorCode, SampleRate, Signal};
    use matches::assert_matches;

    #[test]
    fn set_get_inband_fec() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.inband_fec(), Ok(false));

        encoder
            .set_inband_fec(true)
            .expect("Could not set inband FEC to true.");
        assert_matches!(encoder.inband_fec(), Ok(true));

        encoder
            .set_inband_fec(false)
            .expect("Could not set inband FEC to false.");
        assert_matches!(encoder.inband_fec(), Ok(false));

        encoder
            .enable_inband_fec()
            .expect("Could not set inband FEC to true.");
        assert_matches!(encoder.inband_fec(), Ok(true));

        encoder
            .disable_inband_fec()
            .expect("Could not set inband FEC to false.");
        assert_matches!(encoder.inband_fec(), Ok(false));
    }

    #[test]
    fn set_get_vbr_constraint() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.vbr_constraint(), Ok(true));

        encoder
            .set_vbr_constraint(false)
            .expect("Could not disable VBR constraint.");
        assert_matches!(encoder.vbr_constraint(), Ok(false));

        encoder
            .set_vbr_constraint(true)
            .expect("Could not enable VBR constraint.");
        assert_matches!(encoder.vbr_constraint(), Ok(true));

        encoder
            .enable_vbr_constraint()
            .expect("Could not enable VBR constraint.");
        assert_matches!(encoder.vbr_constraint(), Ok(true));

        encoder
            .disable_vbr_constraint()
            .expect("Could not disable VBR constraint.");
        assert_matches!(encoder.vbr_constraint(), Ok(false));
    }

    #[test]
    fn set_get_vbr() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.vbr(), Ok(true));

        encoder.set_vbr(false).expect("Could not disable VBR.");
        assert_matches!(encoder.vbr(), Ok(false));

        encoder.set_vbr(true).expect("Could not enable VBR.");
        assert_matches!(encoder.vbr(), Ok(true));

        encoder.enable_vbr().expect("Could not enable VBR.");
        assert_matches!(encoder.vbr(), Ok(true));

        encoder.disable_vbr().expect("Could not disable VBR.");
        assert_matches!(encoder.vbr(), Ok(false));
    }

    #[test]
    fn set_get_packet_loss_perc() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.packet_loss_perc(), Ok(0));

        encoder
            .set_packet_loss_perc(10)
            .expect("Could not set packet loss perc to 10%.");
        assert_matches!(encoder.packet_loss_perc(), Ok(10));

        encoder
            .set_packet_loss_perc(100)
            .expect("Could not set packet loss perc to 100%.");
        assert_matches!(encoder.packet_loss_perc(), Ok(100));

        assert_matches!(
            encoder.set_packet_loss_perc(101),
            Err(Error::Opus(ErrorCode::BadArgument))
        );
        assert_matches!(encoder.packet_loss_perc(), Ok(100));
    }

    #[test]
    fn set_get_force_channels() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.force_channels(), Ok(Channels::Auto));

        encoder
            .set_force_channels(Channels::Mono)
            .expect("Could not set force channels to mono.");
        assert_matches!(encoder.force_channels(), Ok(Channels::Mono));

        encoder
            .set_force_channels(Channels::Stereo)
            .expect("Could not set force channels to stereo.");
        assert_matches!(encoder.force_channels(), Ok(Channels::Stereo));

        encoder
            .set_force_channels(Channels::Auto)
            .expect("Could not set force channels to mono.");
        assert_matches!(encoder.force_channels(), Ok(Channels::Auto));
    }

    #[test]
    fn set_get_prediction_disabled() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.prediction_disabled(), Ok(false));

        encoder
            .set_prediction_disabled(true)
            .expect("Could not set prediction disabled to true.");
        assert_matches!(encoder.prediction_disabled(), Ok(true));

        encoder
            .set_prediction_disabled(false)
            .expect("Could not set prediction disabled to false.");
        assert_matches!(encoder.prediction_disabled(), Ok(false));
    }

    #[test]
    fn set_get_signal() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.signal(), Ok(Signal::Auto));

        encoder
            .set_signal(Signal::Music)
            .expect("Could not set signal to music.");
        assert_matches!(encoder.signal(), Ok(Signal::Music));

        encoder
            .set_signal(Signal::Voice)
            .expect("Could not set signal to voice.");
        assert_matches!(encoder.signal(), Ok(Signal::Voice));

        encoder
            .set_signal(Signal::Auto)
            .expect("Could not set signal back to.");
        assert_matches!(encoder.signal(), Ok(Signal::Auto));
    }

    #[test]
    fn encoder_construction() {
        assert_matches!(
            Encoder::new(SampleRate::Hz48000, Channels::Auto, Application::Audio),
            Err(Error::Opus(ErrorCode::BadArgument))
        );

        Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio)
            .expect("Could not create stereo audio encoder");

        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Audio)
            .expect("Could not create mono audio encoder");
    }

    #[test]
    fn encoding() {
        let stereo_encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        // 48000Hz * 1 channel * 20 ms / 1000
        const STEREO_20MS: usize = 48000 * 2 * 20 / 1000;
        let input = [0_i16; STEREO_20MS];
        let mut output = [0; 256];

        let len = stereo_encoder.encode(&input, &mut output).unwrap();
        assert_eq!(&output[..len], &[252, 255, 254]);

        let mono_encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Audio).unwrap();

        // 48000Hz * 1 channel * 20 ms / 1000
        const MONO_20MS: usize = 48000 * 1 * 20 / 1000;
        let input = [0_i16; MONO_20MS];
        let mut output = [0; 256];

        let len = mono_encoder.encode(&input, &mut output).unwrap();
        assert_eq!(&output[..len], &[248, 255, 254]);
    }

    #[test]
    fn set_max_bandwidth() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.max_bandwidth(), Ok(Bandwidth::Fullband));

        let bandwidth_to_set = Bandwidth::Narrowband;

        encoder.set_max_bandwidth(bandwidth_to_set).unwrap();
        let bandwidth_got = encoder.max_bandwidth().unwrap();

        assert_eq!(bandwidth_to_set, bandwidth_got);
    }

    #[test]
    fn get_set_complexity() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        encoder
            .set_complexity(10)
            .expect("Could not set complexity to 10.");
        assert_matches!(encoder.complexity(), Ok(10));

        encoder
            .set_complexity(0)
            .expect("Could not set complexity to 0.");
        assert_matches!(encoder.complexity(), Ok(0));

        assert_matches!(
            encoder.set_complexity(11),
            Err(Error::Opus(ErrorCode::BadArgument))
        );
    }

    #[test]
    fn set_get_application() {
        let application_to_set = Application::Audio;
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, application_to_set).unwrap();
        let current_application = encoder.application().unwrap();
        assert_eq!(current_application, application_to_set);

        let application_to_set = Application::Voip;
        encoder.set_application(application_to_set).unwrap();
        let current_application = encoder.application().unwrap();
        assert_eq!(current_application, application_to_set);

        let application_to_set = Application::LowDelay;
        encoder.set_application(application_to_set).unwrap();
        let current_application = encoder.application().unwrap();
        assert_eq!(current_application, application_to_set);

        let application_to_set = Application::Audio;
        encoder.set_application(application_to_set).unwrap();
        let current_application = encoder.application().unwrap();
        assert_eq!(current_application, application_to_set);
    }

    #[test]
    fn set_get_bitrate() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Audio).unwrap();

        let bitrate = 512000;

        encoder
            .set_bitrate(Bitrate::BitsPerSecond(bitrate))
            .expect("Could not set bitrate to 512000.");

        assert_matches!(encoder.bitrate(), _bitrate);
    }

    #[test]
    fn set_get_dtx() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.dtx(), Ok(false));

        encoder
            .enable_dtx()
            .expect("Could not set dtx to true.");
        assert_matches!(encoder.dtx(), Ok(true));

        encoder
            .disable_dtx()
            .expect("Could not set dtx to false.");
        assert_matches!(encoder.dtx(), Ok(false));
    }

    #[test]
    fn set_get_lsb_depth() {
        let mut encoder =
            Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).unwrap();

        assert_matches!(encoder.lsb_depth(), Ok(24));

        encoder
            .set_lsb_depth(16)
            .expect("Could not set lsb depth to 16.");
        assert_matches!(encoder.lsb_depth(), Ok(16));

        encoder
            .set_lsb_depth(8)
            .expect("Could not set lsb depth to 8.");
        assert_matches!(encoder.lsb_depth(), Ok(8));

        assert_matches!(
            encoder.set_lsb_depth(7),
            Err(Error::Opus(ErrorCode::BadArgument))
        );

        assert_matches!(
            encoder.set_lsb_depth(25),
            Err(Error::Opus(ErrorCode::BadArgument))
        );

        assert_matches!(encoder.lsb_depth(), Ok(8));
    }
}
