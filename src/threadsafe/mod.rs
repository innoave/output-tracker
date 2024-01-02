use crate::inner_listener::{BasicListener, CelledListener};
use crate::inner_tracker::{BasicTracker, CelledTracker};
use crate::tracker_handle::TrackerHandle;
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to obtain lock for tracker")]
    LockTrackerFailed,
    #[error("failed to obtain lock for listener")]
    LockListenerFailed,
}

#[derive(Debug)]
pub struct OutputTracker<M> {
    handle: TrackerHandle,
    inner: ThreadsafeTracker<M>,
    listener: ThreadsafeListener<M>,
}

impl<M> OutputTracker<M> {
    fn new(
        handle: TrackerHandle,
        inner: ThreadsafeTracker<M>,
        listener: ThreadsafeListener<M>,
    ) -> Self {
        Self {
            handle,
            inner,
            listener,
        }
    }

    pub fn stop(&self) -> Result<(), Error> {
        self.listener.remove_tracker(self.handle)
    }

    pub fn clear(&self) -> Result<(), Error> {
        self.inner.clear()
    }

    pub fn output(&self) -> Result<Vec<M>, Error>
    where
        M: Clone,
    {
        self.inner.output()
    }
}

#[derive(Default, Debug)]
pub struct OutputListener<M> {
    inner: ThreadsafeListener<M>,
}

impl<M> OutputListener<M> {
    pub fn new() -> Self {
        Self {
            inner: ThreadsafeListener::new(),
        }
    }
}

impl<M> OutputListener<M>
where
    M: Clone,
{
    pub fn create_tracker(&self) -> Result<OutputTracker<M>, Error> {
        let new_tracker = ThreadsafeTracker::new();
        let handle = self.inner.add_tracker(new_tracker.clone())?;
        Ok(OutputTracker::new(handle, new_tracker, self.inner.clone()))
    }

    pub fn emit(&self, data: M) -> Result<(), Error> {
        self.inner.emit(data)
    }
}

#[derive(Default, Debug, Clone)]
struct ThreadsafeListener<M> {
    cell: Arc<Mutex<BasicListener<M, ThreadsafeTracker<M>>>>,
}

impl<M> ThreadsafeListener<M> {
    fn new() -> Self {
        Self {
            cell: Arc::new(Mutex::new(BasicListener::new())),
        }
    }
}

impl<M> CelledListener<M, ThreadsafeTracker<M>> for ThreadsafeListener<M> {
    type Inner<'a> = MutexGuard<'a, BasicListener<M, ThreadsafeTracker<M>>> where Self: 'a;
    type InnerMut<'a> = MutexGuard<'a, BasicListener<M, ThreadsafeTracker<M>>> where Self: 'a;
    type Error = Error;

    fn listener(&self) -> Result<Self::Inner<'_>, Error> {
        loop {
            match self.cell.try_lock() {
                Ok(listener) => return Ok(listener),
                Err(TryLockError::WouldBlock) => {
                    // try again
                },
                Err(TryLockError::Poisoned(_)) => return Err(Error::LockListenerFailed),
            }
        }
    }

    fn listener_mut(&self) -> Result<Self::InnerMut<'_>, Error> {
        self.listener()
    }
}

#[derive(Debug, Clone)]
struct ThreadsafeTracker<M> {
    cell: Arc<Mutex<BasicTracker<M>>>,
}

impl<M> CelledTracker<M> for ThreadsafeTracker<M> {
    type Inner<'a> = MutexGuard<'a, BasicTracker<M>> where M: 'a;
    type InnerMut<'a> = MutexGuard<'a, BasicTracker<M>> where M: 'a;
    type Error = Error;

    fn new() -> Self {
        Self {
            cell: Arc::new(Mutex::new(BasicTracker::new())),
        }
    }

    fn tracker(&self) -> Result<Self::Inner<'_>, Self::Error> {
        loop {
            match self.cell.try_lock() {
                Ok(tracker) => {
                    return Ok(tracker);
                },
                Err(TryLockError::WouldBlock) => {
                    // try again
                },
                Err(TryLockError::Poisoned(_)) => return Err(Error::LockTrackerFailed),
            }
        }
    }

    fn tracker_mut(&self) -> Result<Self::InnerMut<'_>, Self::Error> {
        self.tracker()
    }
}
