use std::time::Duration;

use exmex::{Express, FlatEx};
use rodio::{source::SeekError, Source};

// TODO: add a sample rate setting based on available rates
const SAMPLE_RATE: u32 = 48000;

#[derive(Clone, Debug)]
pub struct Math {
    func: FlatEx<f32>,
    x: usize,
}

impl Math {
    /// The frequency of the sine.
    #[inline]
    pub fn new(func: FlatEx<f32>) -> Self {
        Self { func, x: 0 }
    }
}

impl Iterator for Math {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.x = self.x.wrapping_add(1);

        self.func
            .eval(&[self.x as f32 / self.sample_rate() as f32])
            .ok()
    }
}

impl Source for Math {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }

    fn try_seek(&mut self, _: Duration) -> Result<(), SeekError> {
        // This is a constant sound, normal seeking would not have any effect.
        // While changing the phase of the sine wave could change how it sounds in
        // combination with another sound (beating) such precision is not the intend
        // of seeking
        Ok(())
    }
}
