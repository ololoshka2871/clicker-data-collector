use std::fmt::Debug;

#[derive(Copy, Clone, Debug)]
pub enum MeasureResult {
    Rk(f32),
    Freq(f32),
}

pub trait ClickerInterface<E: Debug + Send>: Send {
    // read output result from clicker
    fn read(&mut self) -> impl std::future::Future<Output = Result<MeasureResult, E>> + Send;
}
