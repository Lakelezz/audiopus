use crate::{ffi, Channels, MutSignals, Result, TryInto};

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
    pub fn apply<'a>(&mut self, signals: impl TryInto<MutSignals<'a, f32>>) -> Result<()> {
        let mut signals = signals.try_into()?;

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

    #[test]
    fn soft_clip() {
        let mut soft_clip = SoftClip::new(Channels::Stereo);

        let mut signals: Vec<f32> = vec![];
        soft_clip.apply(&mut signals).unwrap();

        signals.push(5.0);
        signals.push(0.2);
        signals.push(-4.0);
        soft_clip.apply(&mut signals).unwrap();

        assert!(signals[0] <= 1.0 && signals[0] >= -1.0);
        assert!(signals[1] <= 1.0 && signals[1] >= -1.0);
        assert!(signals[2] <= 1.0 && signals[2] >= -1.0);
    }
}
