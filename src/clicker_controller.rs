use std::{
    fmt::Debug,
    future::IntoFuture,
    sync::Arc,
    time::{Duration, SystemTime},
};

use futures::future::try_join;
use serde::Serialize;
use tokio::sync::{
    watch::{Receiver, Sender},
    Mutex,
};

use crate::{box_plot::BoxPlot, clicker_interface::ClickerInterface};

#[derive(Serialize, Debug, Clone, Copy)]
pub enum MeasureProcessState {
    Idle,
    Running,
    Interrupted,
    Finished,
}

pub struct ClickerController {
    status_rx: Receiver<MeasureResult>,
    mc_status_rx: Option<Receiver<MeasureProcessStat>>,
    switch_cycles: usize,

    measure_handle: Option<tokio::task::JoinHandle<()>>,
    cancel_tx: Sender<()>,
    cancel_rx: Receiver<()>,
}

#[derive(Debug)]
pub struct MeasureResult {
    pub timestamp: SystemTime,
    pub result: crate::clicker_interface::MeasureResult,
}

#[derive(Serialize, Debug, Clone)]
pub struct MeasureProcessStat {
    pub timestamp: SystemTime,
    pub state: MeasureProcessState,

    pub freqs: Vec<f32>,
    pub rks: Vec<f32>,

    pub freqs_avg: Option<BoxPlot<f32>>,
    pub rks_avg: Option<BoxPlot<f32>>,
}

impl Default for MeasureProcessStat {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now(),
            state: MeasureProcessState::Idle,
            freqs: Vec::new(),
            rks: Vec::new(),
            freqs_avg: None,
            rks_avg: None,
        }
    }
}

unsafe impl Send for MeasureProcessStat {}

impl From<(&[f32], &[f32])> for MeasureProcessStat {
    fn from((freqs, rks): (&[f32], &[f32])) -> Self {
        let freqs_avg = if freqs.is_empty() {
            None
        } else {
            Some(BoxPlot::new(freqs))
        };

        let rks_avg = if rks.is_empty() {
            None
        } else {
            Some(BoxPlot::new(rks))
        };

        Self {
            timestamp: SystemTime::now(),
            state: MeasureProcessState::Running,
            freqs: freqs.to_vec(),
            rks: rks.to_vec(),
            freqs_avg,
            rks_avg,
        }
    }
}

impl ClickerController {
    pub fn new<E: Debug + Send + 'static, C: ClickerInterface<E> + 'static>(
        clicker: C,
        update_interval: Duration,
        switch_cycles: usize,
    ) -> Self {
        let clicker = Arc::new(Mutex::new(clicker));

        let (status_tx, status_rx) = tokio::sync::watch::channel(MeasureResult {
            timestamp: SystemTime::now(),
            result: crate::clicker_interface::MeasureResult::Freq(f32::NAN),
        });

        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(());

        tokio::spawn(read_task(status_tx, clicker.clone(), update_interval));

        Self {
            status_rx,
            mc_status_rx: None,
            switch_cycles,

            measure_handle: None,
            cancel_tx,
            cancel_rx,
        }
    }

    /// Получить экземпляр рессивера обновленя статуса
    pub fn subscribe(&self) -> Receiver<MeasureResult> {
        self.status_rx.clone()
    }

    // Начать измерительный процесс
    pub fn start_mesure(&mut self) -> Result<(), String> {
        match &self.mc_status_rx {
            Some(jh) => {
                match jh.borrow().state {
                    MeasureProcessState::Running | MeasureProcessState::Idle => {
                        return Err("Измерительный процесс уже запущен!".to_string());
                    }
                    _ => (),
                }
                self.mc_status_rx.take();
            }
            _ => (),
        }

        let (mc_status_tx, mc_status_rx) =
            tokio::sync::watch::channel(MeasureProcessStat::default());
        self.mc_status_rx.replace(mc_status_rx);

        self.measure_handle.replace(tokio::spawn(measure_task(
            self.status_rx.clone(),
            mc_status_tx,
            self.switch_cycles,
            self.cancel_rx.clone(),
        )));

        Ok(())
    }

    // Прервать измерительный процесс
    pub async fn interrupt_mesure(&mut self) {
        if let Some(h) = self.measure_handle.take() {
            self.cancel_tx.send(()).ok();
            self.mc_status_rx.take();
            let _ = h.await;
        }
    }

    // Получить экземпляр рессивера обновленя статуса измерительного процесса
    pub fn subscribe_measure_status(&self) -> Option<Receiver<MeasureProcessStat>> {
        self.mc_status_rx.clone()
    }
}

async fn read_task<E: Debug + Send, C: ClickerInterface<E>>(
    status_tx: Sender<MeasureResult>,
    clicker: Arc<Mutex<C>>,
    update_interval: Duration,
) {
    const TRYS: usize = 3;

    let mut fails = 0;
    loop {
        match clicker.lock().await.read().await {
            Ok(result) => {
                fails = 0;
                let res: MeasureResult = MeasureResult {
                    timestamp: SystemTime::now(),
                    result,
                };
                tracing::trace!("Read from clicker: {:?}", res);
                status_tx.send(res).ok();
            }
            Err(e) => {
                fails += 1;
                if fails >= TRYS {
                    panic!("Fatal error reading from clicker: {:?}", e);
                } else {
                    tracing::error!("Error reading from clicker: {:?}", e);
                }
            }
        }
        tokio::time::sleep(update_interval).await;
    }
}

async fn measure_task(
    mut status_rx: Receiver<MeasureResult>,
    mc_status_tx: Sender<MeasureProcessStat>,
    switch_cycles: usize,
    mut cancel_rx: Receiver<()>,
) {
    let mut freqs = Vec::new();
    let mut rks = Vec::new();
    let mut swiches_count = 0;
    let mut prev_mode_freq = matches!(
        status_rx.borrow().result,
        crate::clicker_interface::MeasureResult::Freq(_)
    );
    let mut wait_first_switch = true;

    cancel_rx.mark_unchanged(); // нужно сбросить, чтобы не было циклической отмены

    let send = |result: MeasureProcessStat, swiches_count: usize| {
        tracing::trace!(
            "Measure in {:?}: sw_c={} f={:?}, rk={:?}",
            result.state,
            swiches_count,
            result.freqs_avg,
            result.rks_avg
        );
        mc_status_tx.send(result).ok();
    };

    loop {
        tokio::select! {
            res = status_rx.changed() => {
                match res {
                    Ok(_) => {
                        let res = status_rx.borrow();

                        if prev_mode_freq
                            != matches!(res.result, crate::clicker_interface::MeasureResult::Freq(_))
                        {
                            wait_first_switch = false;
                            swiches_count += 1;
                            prev_mode_freq = !prev_mode_freq;
                        } else if wait_first_switch {
                            continue;
                        }

                        match res.result {
                            crate::clicker_interface::MeasureResult::Freq(f) => {
                                freqs.push(f);
                            }
                            crate::clicker_interface::MeasureResult::Rk(r) => {
                                rks.push(r);
                            }
                        }

                        let mut result = MeasureProcessStat::from((freqs.as_slice(), rks.as_slice()));

                        if swiches_count > switch_cycles * 2 {
                            result.state = MeasureProcessState::Finished;
                            send(result, swiches_count);
                            break;
                        } else {
                            send(result, swiches_count);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error reading from status_rx: {:?}", e);
                        let mut result = MeasureProcessStat::from((freqs.as_slice(), rks.as_slice()));
                        result.state = MeasureProcessState::Interrupted;
                        send(result, swiches_count);
                        break;
                    }
                }
            }
            res = cancel_rx.changed() => {
                // cancel
                match res {
                    Ok(_) => {
                        tracing::warn!("Measure was canceled");
                        let mut result = MeasureProcessStat::from((freqs.as_slice(), rks.as_slice()));
                        result.state = MeasureProcessState::Interrupted;
                        send(result, swiches_count);
                        break;
                    }
                    Err(e) => {
                        panic!("Error reading from cancel_rx: {:?}", e);
                    }
                }
            }
        }
    }
}
