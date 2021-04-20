use crate::{ffi, Channels, MutSignals, Result};

#[derive(Clone, Debug)]
pub struct SoftClip {
    channels: Channels,
    memory: [f32; 2],
}

impl SoftClip {
    pub fn new(channels: Channels) -> Self {
        Self {
            channels,
            memory: [0.0; 2],
        }
    }

    /// Opus applies soft-clipping to bring a f32 signal within the
    /// [-1,1] range.
    pub fn apply(&mut self, mut signals: MutSignals<'_, f32>) -> Result<()> {
        unsafe {
            ffi::opus_pcm_soft_clip(
                signals.as_mut_ptr(),
                signals.i32_len() / self.channels as i32,
                self.channels as i32,
                self.memory.as_mut_ptr(),
            )
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SoftClip;
    use crate::Channels;
    use std::convert::TryInto;

    #[test]
    fn soft_clip() {
        let mut soft_clip = SoftClip::new(Channels::Stereo);

        let mut signals: Vec<f32> = vec![];
        soft_clip.apply((&mut signals).try_into().unwrap()).unwrap();

        signals.push(5.0);
        signals.push(-5000.3453);
        soft_clip.apply((&mut signals).try_into().unwrap()).unwrap();

        assert!(signals[0] <= 1.0 && signals[0] >= -1.0);
        assert!(signals[1] <= 1.0 && signals[1] >= -1.0);
    }
}
