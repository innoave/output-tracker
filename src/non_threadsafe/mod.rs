use crate::inner_subject::{BasicSubject, CelledSubject};
use crate::inner_tracker::{BasicTracker, CelledTracker};
use crate::non_threadsafe::Error::{BorrowMutTrackerFailed, BorrowTrackerFailed};
use crate::tracker_handle::TrackerHandle;
use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use std::rc::Rc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to obtain immutable borrow of tracker, reason: {0}")]
    BorrowTrackerFailed(BorrowError),
    #[error("failed to obtain mutable borrow of tracker, reason: {0}")]
    BorrowMutTrackerFailed(BorrowMutError),
    #[error("failed to obtain immutable borrow of subject, reason: {0}")]
    BorrowSubjectFailed(BorrowError),
    #[error("failed to obtain mutable borrow of subject, reason: {0}")]
    BorrowMutSubjectFailed(BorrowMutError),
}

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
    inner: NonThreadsafeSubject<M>,
}

impl<M> OutputSubject<M> {
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
    pub fn create_tracker(&self) -> Result<OutputTracker<M>, Error> {
        let new_tracker = NonThreadsafeTracker::new();
        let handle = self.inner.add_tracker(new_tracker.clone())?;
        Ok(OutputTracker::new(handle, new_tracker, self.inner.clone()))
    }

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
    type Inner<'a> = Ref<'a, BasicSubject<M, NonThreadsafeTracker<M>>> where M: 'a;
    type InnerMut<'a> = RefMut<'a, BasicSubject<M, NonThreadsafeTracker<M>>> where M: 'a;
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
    type Inner<'a> = Ref<'a, BasicTracker<M>> where Self: 'a;
    type InnerMut<'a> = RefMut<'a, BasicTracker<M>> where Self: 'a;
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
