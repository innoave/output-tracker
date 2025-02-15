//! Non-threadsafe variant of [`OutputTracker`] and [`OutputSubject`].
//!
//! For an example on how to use it see the crate level documentation.

use crate::inner_subject::{BasicSubject, CelledSubject};
use crate::inner_tracker::{BasicTracker, CelledTracker};
use crate::non_threadsafe::Error::{BorrowMutTrackerFailed, BorrowTrackerFailed};
use crate::tracker_handle::TrackerHandle;
use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use std::rc::Rc;

/// Error type for the non-threadsafe [`OutputTracker`] and [`OutputSubject`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to obtain an immutable borrow of the tracker.
    #[error("failed to obtain an immutable borrow of the tracker, reason: {0}")]
    BorrowTrackerFailed(BorrowError),
    /// Failed to obtain a mutable borrow of the tracker.
    #[error("failed to obtain a mutable borrow of the tracker, reason: {0}")]
    BorrowMutTrackerFailed(BorrowMutError),
    /// Failed to obtain an immutable borrow of the subject.
    #[error("failed to obtain an immutable borrow of the subject, reason: {0}")]
    BorrowSubjectFailed(BorrowError),
    /// Failed to obtain a mutable borrow of the subject.
    #[error("failed to obtain a mutable borrow of the subject, reason: {0}")]
    BorrowMutSubjectFailed(BorrowMutError),
}

/// Collects state data or action data of any kind.
///
/// This is the non-threadsafe variant.
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
    inner: NonThreadsafeTracker<M>,
    subject: NonThreadsafeSubject<M>,
}

impl<M> OutputTracker<M> {
    const fn new(
        handle: TrackerHandle,
        inner: NonThreadsafeTracker<M>,
        subject: NonThreadsafeSubject<M>,
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
/// This is the non-threadsafe variant.
///
/// New [`OutputTracker`]s can be created by calling the
/// [`create_tracker()`][OutputSubject::create_tracker] function.
///
/// The [`emit(data)`][OutputSubject::emit] function emits data to all trackers,
/// that have been created for this subject and are not stopped yet.
#[derive(Default, Debug, Clone)]
pub struct OutputSubject<M> {
    inner: NonThreadsafeSubject<M>,
}

impl<M> OutputSubject<M> {
    /// Constructs a new [`OutputSubject`].
    ///
    /// A new subject does nothing unless one or more trackers have been
    /// created.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: NonThreadsafeSubject::new(),
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
        let new_tracker = NonThreadsafeTracker::new();
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
struct NonThreadsafeSubject<M> {
    cell: Rc<RefCell<BasicSubject<M, NonThreadsafeTracker<M>>>>,
}

impl<M> NonThreadsafeSubject<M> {
    fn new() -> Self {
        Self {
            cell: Rc::new(RefCell::new(BasicSubject::new())),
        }
    }
}

impl<M> CelledSubject<M, NonThreadsafeTracker<M>> for NonThreadsafeSubject<M> {
    type Inner<'a>
        = Ref<'a, BasicSubject<M, NonThreadsafeTracker<M>>>
    where
        M: 'a;
    type InnerMut<'a>
        = RefMut<'a, BasicSubject<M, NonThreadsafeTracker<M>>>
    where
        M: 'a;
    type Error = Error;

    fn subject(&self) -> Result<Self::Inner<'_>, Error> {
        self.cell.try_borrow().map_err(Error::BorrowSubjectFailed)
    }

    fn subject_mut(&self) -> Result<Self::InnerMut<'_>, Error> {
        self.cell
            .try_borrow_mut()
            .map_err(Error::BorrowMutSubjectFailed)
    }
}

#[derive(Debug, Clone)]
struct NonThreadsafeTracker<M> {
    cell: Rc<RefCell<BasicTracker<M>>>,
}

impl<M> CelledTracker<M> for NonThreadsafeTracker<M> {
    type Inner<'a>
        = Ref<'a, BasicTracker<M>>
    where
        Self: 'a;
    type InnerMut<'a>
        = RefMut<'a, BasicTracker<M>>
    where
        Self: 'a;
    type Error = Error;

    fn new() -> Self {
        Self {
            cell: Rc::new(RefCell::new(BasicTracker::new())),
        }
    }

    fn tracker(&self) -> Result<Self::Inner<'_>, Self::Error> {
        self.cell.try_borrow().map_err(BorrowTrackerFailed)
    }

    fn tracker_mut(&self) -> Result<Self::InnerMut<'_>, Self::Error> {
        self.cell.try_borrow_mut().map_err(BorrowMutTrackerFailed)
    }
}

#[cfg(test)]
mod tests;
