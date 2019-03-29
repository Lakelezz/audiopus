use crate::{Error, SampleRate};

pub use self::{
    decoder::{size, Decoder},
    encoder::Encoder,
};

mod decoder;
mod encoder;

/// A set of methods that both `Encoder` and `Decoder` have implemented.
///
/// **Info**:
/// This does not include `set_sample_rate` as it returns unimplemented on
/// [`Decoder`].
///
/// [`Decoder`]: decoder/struct.Decoder.html
pub trait GenericCtl {
    fn final_range(&self) -> Result<u32, Error>;

    fn phase_inversion_disabled(&self) -> Result<bool, Error>;
    fn set_phase_inversion_disabled(&mut self, disabled: bool) -> Result<(), Error>;

    fn sample_rate(&self) -> Result<SampleRate, Error>;

    fn reset_state(&mut self) -> Result<(), Error>;
}
