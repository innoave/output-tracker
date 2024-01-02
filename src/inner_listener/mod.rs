use crate::inner_tracker::CelledTracker;
use crate::tracker_handle::TrackerHandle;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice;

pub(crate) trait CelledListener<M, T> {
    type Inner<'a>: Deref<Target = BasicListener<M, T>>
    where
        Self: 'a;
    type InnerMut<'a>: DerefMut<Target = BasicListener<M, T>>
    where
        Self: 'a;
    type Error: std::error::Error;

    fn listener(&self) -> Result<Self::Inner<'_>, Self::Error>;

    fn listener_mut(&self) -> Result<Self::InnerMut<'_>, Self::Error>;

    fn add_tracker(&self, tracker: T) -> Result<TrackerHandle, Self::Error>
    where
        T: CelledTracker<M>,
    {
        self.listener_mut()
            .map(|mut listener| listener.add_tracker(tracker))
    }

    fn remove_tracker(&self, tracker: TrackerHandle) -> Result<(), Self::Error> {
        self.listener_mut()
            .map(|mut listener| listener.remove_tracker(tracker))
    }

    fn emit(&self, data: M) -> Result<(), Self::Error>
    where
        M: Clone,
        T: CelledTracker<M>,
        Self::Error: From<<T as CelledTracker<M>>::Error>,
    {
        for tracker in self.listener()?.trackers() {
            tracker.track(data.clone())?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct BasicListener<M, T> {
    _data: PhantomData<M>,
    trackers: Vec<(TrackerHandle, T)>,
}

impl<M, T> Default for BasicListener<M, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M, T> BasicListener<M, T> {
    pub fn new() -> Self {
        Self {
            _data: PhantomData,
            trackers: Vec::new(),
        }
    }

    pub fn trackers(&self) -> Trackers<'_, T> {
        Trackers::new(self.trackers.iter())
    }

    pub fn add_tracker(&mut self, tracker: T) -> TrackerHandle {
        let handle = TrackerHandle::new();
        self.trackers.push((handle, tracker));
        handle
    }

    pub fn remove_tracker(&mut self, tracker: TrackerHandle) {
        let found_index = self.trackers.iter().position(|&(it, _)| it == tracker);
        found_index.iter().for_each(|&idx| {
            let _ = self.trackers.remove(idx);
        })
    }
}

pub(crate) struct Trackers<'a, T> {
    inner: slice::Iter<'a, (TrackerHandle, T)>,
}

impl<'a, T> Trackers<'a, T> {
    fn new(trackers: slice::Iter<'a, (TrackerHandle, T)>) -> Self {
        Self { inner: trackers }
    }
}

impl<'a, T> Iterator for Trackers<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, tracker)| tracker)
    }
}
