use crate::inner_subject::{BasicSubject, CelledSubject};
use crate::inner_tracker::{BasicTracker, CelledTracker};
use crate::tracker_handle::TrackerHandle;
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to obtain lock for tracker")]
    LockTrackerFailed,
    #[error("failed to obtain lock for subject")]
    LockSubjectFailed,
}

#[derive(Debug)]
pub struct OutputTracker<M> {
    handle: TrackerHandle,
    inner: ThreadsafeTracker<M>,
    subject: ThreadsafeSubject<M>,
}

impl<M> OutputTracker<M> {
    fn new(
        handle: TrackerHandle,
        inner: ThreadsafeTracker<M>,
        subject: ThreadsafeSubject<M>,
    ) -> Self {
        Self {
            handle,
            inner,
            subject,
        }
    }

    pub fn stop(&self) -> Result<(), Error> {
        self.subject.remove_tracker(self.handle)
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

#[derive(Default, Debug, Clone)]
pub struct OutputSubject<M> {
    inner: ThreadsafeSubject<M>,
}

impl<M> OutputSubject<M> {
    pub fn new() -> Self {
        Self {
            inner: ThreadsafeSubject::new(),
        }
    }
}

impl<M> OutputSubject<M>
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
struct ThreadsafeSubject<M> {
    cell: Arc<Mutex<BasicSubject<M, ThreadsafeTracker<M>>>>,
}

impl<M> ThreadsafeSubject<M> {
    fn new() -> Self {
        Self {
            cell: Arc::new(Mutex::new(BasicSubject::new())),
        }
    }
}

impl<M> CelledSubject<M, ThreadsafeTracker<M>> for ThreadsafeSubject<M> {
    type Inner<'a> = MutexGuard<'a, BasicSubject<M, ThreadsafeTracker<M>>> where Self: 'a;
    type InnerMut<'a> = MutexGuard<'a, BasicSubject<M, ThreadsafeTracker<M>>> where Self: 'a;
    type Error = Error;

    fn subject(&self) -> Result<Self::Inner<'_>, Error> {
        loop {
            match self.cell.try_lock() {
                Ok(subject) => return Ok(subject),
                Err(TryLockError::WouldBlock) => {
                    // try again
                },
                Err(TryLockError::Poisoned(_)) => return Err(Error::LockSubjectFailed),
            }
        }
    }

    fn subject_mut(&self) -> Result<Self::InnerMut<'_>, Error> {
        self.subject()
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
