use std::{
    future::Future,
    sync::{Arc, Mutex},
    task::Waker,
    time::Duration,
};
use tokio::{
    sync::oneshot::{self, error::TryRecvError},
    time::sleep,
};

use crate::{Guard, RunningGuard};

struct TokioGuard<AT, ST, F> {
    resource_state: AT,
    stopped_resource_state: ST,
    guard: F,
    sleep_duration: Duration,
}

impl<AT, ST, T, F> Guard for TokioGuard<AT, ST, F>
where
    T: Future<Output = bool> + Send,
    F: Fn() -> T + Send + Sync + 'static,
{
    type ResourceState = AT;
    type RunningGuard = TokioRunningGuard<ST>;

    fn start(self) -> (Self::ResourceState, Self::RunningGuard) {
        let (tx, mut rx) = oneshot::channel();
        let guard = TokioRunningGuard {
            stopped_resource_state: Some(self.stopped_resource_state),
            waker: Arc::new(Mutex::new(None)),
            ok: Arc::new(Mutex::new(true)),
            stop_sender: Some(tx),
        };

        let waker = guard.waker.clone();
        let ok = guard.ok.clone();
        let guard_f = self.guard;
        let duration = self.sleep_duration;
        tokio::spawn(async move {
            loop {
                match rx.try_recv() {
                    Ok(_) => break,
                    Err(TryRecvError::Closed) => break,
                    Err(TryRecvError::Empty) => {
                        if guard_f().await {
                            *ok.lock().unwrap() = false;
                            let waker_l = waker.lock().unwrap();
                            if let Some(ref waker) = *waker_l {
                                waker.wake_by_ref();
                            }
                        }
                    }
                }

                sleep(duration).await;
            }
        });

        (self.resource_state, guard)
    }
}

struct TokioRunningGuard<ST> {
    stopped_resource_state: Option<ST>,
    waker: Arc<Mutex<Option<Waker>>>,
    ok: Arc<Mutex<bool>>,
    stop_sender: Option<oneshot::Sender<()>>,
}

impl<ST> RunningGuard for TokioRunningGuard<ST> {
    type StoppedResourceState = ST;

    fn is_safe(&self) -> bool {
        self.ok.lock().unwrap().clone()
    }

    fn set_waker(&mut self, waker: Waker) {
        let mut waker_lock = self.waker.lock().unwrap();
        *waker_lock = Some(waker);
    }

    unsafe fn stop(&mut self) -> Self::StoppedResourceState {
        if let Some(sender) = self.stop_sender.take() {
            sender.send(()).expect("Failed to send stop signal");
        }

        self.stopped_resource_state.take().unwrap()
    }
}

pub fn new_tokio_guard<F, T, AT, ST>(
    check: F,
    sleep_duration: Duration,
    resource_state: AT,
    stopped_resource_state: ST,
) -> impl Guard<ResourceState = AT, RunningGuard = impl RunningGuard<StoppedResourceState = ST>>
where
    T: Future<Output = bool> + Send,
    F: Fn() -> T + Send + Sync + 'static,
{
    TokioGuard {
        resource_state: resource_state,
        stopped_resource_state: stopped_resource_state,
        guard: check,
        sleep_duration,
    }
}
