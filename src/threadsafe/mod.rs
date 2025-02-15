//! Threadsafe variant of [`OutputTracker`] and [`OutputSubject`].
//!
//! For an example on how to use it see the crate level documentation.

use crate::inner_subject::{BasicSubject, CelledSubject};
use crate::inner_tracker::{BasicTracker, CelledTracker};
use crate::tracker_handle::TrackerHandle;
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

/// Error type for the threadsafe [`OutputTracker`] and [`OutputSubject`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to obtain a lock for the tracker.
    #[error("failed to obtain a lock for the tracker")]
    LockTrackerFailed,
    /// Failed to obtain a lock for the subject.
    #[error("failed to obtain a lock for the subject")]
    LockSubjectFailed,
}

/// A struct that collects state data or action data of any kind.
///
/// This is the threadsafe variant.
///
/// The tracked data can be read any time and as often as needed by calling the
/// [`output()`][OutputTracker::output]. Each time the output is read, all data
/// collected so far are returned. To track only new data emitted after the last
/// read of the output, the [`clear()`][OutputTracker::clear] function should be
/// called.
///
/// The tracker can be deactivated by calling the [`stop()`][OutputTracker::stop]
/// function to stop it from collecting data. Once stopped the tracker can not
/// be activated again.
#[derive(Debug)]
pub struct OutputTracker<M> {
    handle: TrackerHandle,
    inner: ThreadsafeTracker<M>,
    subject: ThreadsafeSubject<M>,
}

impl<M> OutputTracker<M> {
    const fn new(
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

    /// Stops this tracker.
    ///
    /// After stopping a tracker it no longer tracks emitted data. Once a
    /// tracker is stopped it can not be activated again.
    pub fn stop(&self) -> Result<(), Error> {
        self.subject.remove_tracker(self.handle)
    }

    /// Clears the data this tracker has been collected so far.
    ///
    /// After clearing a tracker it still tracks any data which is emitted after
    /// this clear function has been called.
    pub fn clear(&self) -> Result<(), Error> {
        self.inner.clear()
    }

    /// Returns the data collected by this tracker so far.
    ///
    /// Each time this function is called it returns all data collected since
    /// the tracker has been created or since the last call to of the
    /// [`clear()`][OutputTracker::clear] function. To track only data that are
    /// emitted after the last time the output was read, the
    /// [`clear()`][OutputTracker::clear] should be called after the output has
    /// been read.
    pub fn output(&self) -> Result<Vec<M>, Error>
    where
        M: Clone,
    {
        self.inner.output()
    }
}

/// Holds created [`OutputTracker`]s and emits data to all known trackers.
///
/// This is the threadsafe variant.
///
/// New [`OutputTracker`]s can be created by calling the
/// [`create_tracker()`][OutputSubject::create_tracker] function.
///
/// The [`emit(data)`][OutputSubject::emit] function emits data to all trackers,
/// that have been created for this subject and are not stopped yet.
#[derive(Default, Debug, Clone)]
pub struct OutputSubject<M> {
    inner: ThreadsafeSubject<M>,
}

impl<M> OutputSubject<M> {
    /// Constructs a new [`OutputSubject`].
    ///
    /// A new subject does nothing unless one or more trackers have been
    /// created.
    #[must_use]
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
    /// Creates a new [`OutputTracker`] and registers it to be ready to track
    /// emitted data.
    pub fn create_tracker(&self) -> Result<OutputTracker<M>, Error> {
        let new_tracker = ThreadsafeTracker::new();
        let handle = self.inner.add_tracker(new_tracker.clone())?;
        Ok(OutputTracker::new(handle, new_tracker, self.inner.clone()))
    }

    /// Emits given data to all active [`OutputTracker`]s.
    ///
    /// Stopped [`OutputTracker`]s do not receive any emitted data.
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
    type Inner<'a>
        = MutexGuard<'a, BasicSubject<M, ThreadsafeTracker<M>>>
    where
        Self: 'a;
    type InnerMut<'a>
        = MutexGuard<'a, BasicSubject<M, ThreadsafeTracker<M>>>
    where
        Self: 'a;
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
    type Inner<'a>
        = MutexGuard<'a, BasicTracker<M>>
    where
        M: 'a;
    type InnerMut<'a>
        = MutexGuard<'a, BasicTracker<M>>
    where
        M: 'a;
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

#[cfg(test)]
mod tests;
