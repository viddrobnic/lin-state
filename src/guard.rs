use crate::resource::Resource;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

#[derive(Debug)]
pub enum GuardOutput<T> {
    Ok(T),
    InvalidState,
}

pub trait Guard {
    type ResourceState;
    type RunningGuard: RunningGuard;

    fn start(self) -> (Self::ResourceState, Self::RunningGuard);
}

pub trait RunningGuard {
    type StoppedResourceState;

    fn is_safe(&self) -> bool;
    fn set_waker(&mut self, waker: Waker);

    unsafe fn stop(&mut self) -> Self::StoppedResourceState;
}

pub fn with_guard<T, S, F, G, RG, FUT>(
    guard: G,
    initial_state: S,
    fut_const: F,
) -> impl Future<Output = (S, RG::StoppedResourceState, GuardOutput<T>)>
where
    S: Resource,
    RG: RunningGuard,
    G: Guard<RunningGuard = RG>,
    FUT: Future<Output = T>,
    F: FnOnce(S, G::ResourceState) -> FUT,
{
    let inner_state = unsafe {
        let mut inner_state = initial_state.clone_state();
        inner_state.set_cleanup_enabled(false);
        inner_state
    };

    let (guard_resource, running_guard) = guard.start();

    let future = fut_const(inner_state, guard_resource);
    GuardExecutor {
        running_guard,
        future,
        initial_state: Some(initial_state),
    }
}

struct GuardExecutor<RG, S, F> {
    running_guard: RG,
    future: F,
    initial_state: Option<S>,
}

impl<T, RG, S, F> Future for GuardExecutor<RG, S, F>
where
    RG: RunningGuard,
    F: Future<Output = T>,
    S: Resource,
{
    type Output = (S, RG::StoppedResourceState, GuardOutput<T>);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = cx.waker().clone();
        // Safe, because guard is not considered pinned.
        unsafe {
            self.as_mut()
                .get_unchecked_mut()
                .running_guard
                .set_waker(waker);
        }

        if !self.running_guard.is_safe() {
            // Safe, because:
            // - guard is not considered pinned,
            // - we return `Poll::Ready`, therefore the guard is stopped only once,
            // - `initial_state` is not considered pinned,
            // - we return `Poll::Ready`, therefore `initial_state` is taken only once.
            unsafe {
                let original_state = self.as_mut().get_unchecked_mut().running_guard.stop();
                let initial_state = self
                    .as_mut()
                    .get_unchecked_mut()
                    .initial_state
                    .take()
                    .unwrap();
                return Poll::Ready((initial_state, original_state, GuardOutput::InvalidState));
            }
        }

        // Safe, because fut is pinned since self is pinned.
        let fut = unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.future) };
        match fut.poll(cx) {
            Poll::Ready(val) => {
                // Safe because of the same reasons as above, when checking if guard is still safe.
                unsafe {
                    let original_state = self.as_mut().get_unchecked_mut().running_guard.stop();
                    let initial_state = self
                        .as_mut()
                        .get_unchecked_mut()
                        .initial_state
                        .take()
                        .unwrap();
                    Poll::Ready((initial_state, original_state, GuardOutput::Ok(val)))
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
