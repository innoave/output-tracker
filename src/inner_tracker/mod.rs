use std::ops::{Deref, DerefMut};

pub trait Tracker<M> {
    fn track(&mut self, data: M);
}

pub trait CelledTracker<M> {
    type Inner<'a>: Deref<Target = BasicTracker<M>>
    where
        Self: 'a;
    type InnerMut<'a>: DerefMut<Target = BasicTracker<M>>
    where
        Self: 'a;
    type Error: std::error::Error;

    fn new() -> Self;

    fn tracker(&self) -> Result<Self::Inner<'_>, Self::Error>;

    fn tracker_mut(&self) -> Result<Self::InnerMut<'_>, Self::Error>;

    fn output(&self) -> Result<Vec<M>, Self::Error>
    where
        M: Clone,
    {
        self.tracker().map(|tracker| tracker.output().to_vec())
    }

    fn clear(&self) -> Result<(), Self::Error> {
        self.tracker_mut().map(|mut tracker| tracker.clear())
    }

    fn track(&self, data: M) -> Result<(), Self::Error> {
        self.tracker_mut().map(|mut tracker| tracker.track(data))
    }
}

#[derive(Debug)]
pub struct BasicTracker<M> {
    tracked: Vec<M>,
}

impl<M> BasicTracker<M> {
    pub const fn new() -> Self {
        Self {
            tracked: Vec::new(),
        }
    }

    pub fn output(&self) -> &[M]
    where
        M: Clone,
    {
        &self.tracked
    }

    pub fn clear(&mut self) {
        self.tracked.clear();
    }
}

impl<M> Tracker<M> for BasicTracker<M> {
    fn track(&mut self, data: M) {
        self.tracked.push(data);
    }
}
