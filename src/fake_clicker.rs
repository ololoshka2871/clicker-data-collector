use std::time::{Duration, SystemTime};

use rand::distributions::Distribution;
use rand_distr::Normal;

use crate::clicker_interface::ClickerInterface;

#[derive(Debug)]
pub struct NoError;

pub struct FakeClicker {
    initial: SystemTime,
    switch_duration: Duration,
    distribution_f: Normal<f32>,
    distribution_rk: Normal<f32>,
}

impl ClickerInterface<NoError> for FakeClicker {
    async fn read(&mut self) -> Result<crate::clicker_interface::MeasureResult, NoError> {
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
                self.distribution_rk.sample(&mut rng),
            ))
        } else {
            Ok(crate::clicker_interface::MeasureResult::Freq(
                self.distribution_f.sample(&mut rng),
            ))
        }
    }
}

impl FakeClicker {
    pub fn new(switch_duration: Duration) -> Self {
        const FREQ_CENTER: f32 = 32760.0;
        const RK_CENTER: f32 = 50.0;

        Self {
            initial: SystemTime::now(),
            switch_duration,
            distribution_f: Normal::new(FREQ_CENTER, 5.0).unwrap(),
            distribution_rk: Normal::new(RK_CENTER, 10.0).unwrap(),
        }
    }
}
