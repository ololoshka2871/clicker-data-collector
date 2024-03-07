use std::time::{Duration, SystemTime};

use crate::clicker_interface::ClickerInterface;

#[derive(Debug)]
pub struct NoError;

pub struct FakeClicker {
    initial: SystemTime,
    switch_duration: Duration,
}

impl ClickerInterface<NoError> for FakeClicker {
    async fn read(&mut self) -> Result<crate::clicker_interface::MeasureResult, NoError> {
        use rand::Rng;

        const FREQ_CENTER: f32 = 32760.0;
        const RK_MIN: f32 = 20.0;

        let mut rng = rand::thread_rng();

        if SystemTime::now()
            .duration_since(self.initial)
            .unwrap()
            .as_secs()
            / self.switch_duration.as_secs()
            % 2
            == 0
        {
            Ok(crate::clicker_interface::MeasureResult::Rk(
                RK_MIN + rng.gen_range(-10.0..10.0),
            ))
        } else {
            Ok(crate::clicker_interface::MeasureResult::Freq(
                FREQ_CENTER + rng.gen_range(0.0..100.0),
            ))
        }
    }
}

impl FakeClicker {
    pub fn new(switch_duration: Duration) -> Self {
        Self {
            initial: SystemTime::now(),
            switch_duration,
        }
    }
}
