use crate::inner_listener::{BasicListener, CelledListener};
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
    #[error("failed to obtain immutable borrow of listener, reason: {0}")]
    BorrowListenerFailed(BorrowError),
    #[error("failed to obtain mutable borrow of listener, reason: {0}")]
    BorrowMutListenerFailed(BorrowMutError),
}

#[derive(Debug)]
pub struct OutputTracker<M> {
    handle: TrackerHandle,
    inner: NonThreadsafeTracker<M>,
    listener: NonThreadsafeListener<M>,
}

impl<M> OutputTracker<M> {
    fn new(
        handle: TrackerHandle,
        inner: NonThreadsafeTracker<M>,
        listener: NonThreadsafeListener<M>,
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

#[derive(Default, Debug, Clone)]
pub struct OutputListener<M> {
    inner: NonThreadsafeListener<M>,
}

impl<M> OutputListener<M> {
    pub fn new() -> Self {
        Self {
            inner: NonThreadsafeListener::new(),
        }
    }
}

impl<M> OutputListener<M>
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
struct NonThreadsafeListener<M> {
    cell: Rc<RefCell<BasicListener<M, NonThreadsafeTracker<M>>>>,
}

impl<M> NonThreadsafeListener<M> {
    fn new() -> Self {
        Self {
            cell: Rc::new(RefCell::new(BasicListener::new())),
        }
    }
}

impl<M> CelledListener<M, NonThreadsafeTracker<M>> for NonThreadsafeListener<M> {
    type Inner<'a> = Ref<'a, BasicListener<M, NonThreadsafeTracker<M>>> where M: 'a;
    type InnerMut<'a> = RefMut<'a, BasicListener<M, NonThreadsafeTracker<M>>> where M: 'a;
    type Error = Error;

    fn listener(&self) -> Result<Self::Inner<'_>, Error> {
        self.cell.try_borrow().map_err(Error::BorrowListenerFailed)
    }

    fn listener_mut(&self) -> Result<Self::InnerMut<'_>, Error> {
        self.cell
            .try_borrow_mut()
            .map_err(Error::BorrowMutListenerFailed)
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
