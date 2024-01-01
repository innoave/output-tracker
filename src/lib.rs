#![deny(unsafe_code, unstable_features)]
#![warn(
    bare_trait_objects,
    deprecated,
    explicit_outlives_requirements,
    noop_method_call,
    rust_2018_idioms,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications
)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
//#![warn(missing_docs)] //TODO uncomment eventually
//#![warn(variant_size_differences)] // enable when working on performance
//#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static HANDLE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TrackerHandle(u64);

impl TrackerHandle {
    fn new() -> Self {
        Self(HANDLE_ID.fetch_add(1, Ordering::AcqRel))
    }
}

#[derive(Debug)]
pub struct OutputTracker<T> {
    inner: Rc<RefCell<InnerTracker<T>>>,
    listener: Rc<RefCell<InnerListener<T>>>,
}

impl<T> OutputTracker<T> {
    fn new(inner: Rc<RefCell<InnerTracker<T>>>, listener: Rc<RefCell<InnerListener<T>>>) -> Self {
        Self { inner, listener }
    }

    pub fn stop(&self) {
        self.listener
            .borrow_mut()
            .remove_tracker(self.inner.clone());
    }

    pub fn output(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.inner.borrow().output()
    }

    pub fn clear(&mut self) {
        self.inner.borrow_mut().clear();
    }
}

#[derive(Debug)]
struct InnerTracker<T> {
    handle: TrackerHandle,
    tracked: Vec<T>,
}

impl<T> InnerTracker<T> {
    fn new(handle: TrackerHandle) -> Self {
        Self {
            handle,
            tracked: Vec::new(),
        }
    }

    fn handle(&self) -> TrackerHandle {
        self.handle
    }

    fn output(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.tracked.clone()
    }

    fn clear(&mut self) {
        self.tracked.clear();
    }

    fn push(&mut self, data: T) {
        self.tracked.push(data);
    }
}

#[derive(Default, Debug)]
pub struct OutputListener<T> {
    inner: Rc<RefCell<InnerListener<T>>>,
}

impl<T> OutputListener<T> {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(InnerListener::<T>::new())),
        }
    }

    pub fn create_tracker(&self) -> OutputTracker<T> {
        let new_tracker = Rc::new(RefCell::new(InnerTracker::new(TrackerHandle::new())));
        self.inner.borrow_mut().add_tracker(new_tracker.clone());
        OutputTracker::new(new_tracker, self.inner.clone())
    }

    pub fn emit(&self, data: T)
    where
        T: Clone,
    {
        self.inner.borrow_mut().emit(data);
    }
}

#[derive(Default, Debug)]
struct InnerListener<T> {
    trackers: Vec<Rc<RefCell<InnerTracker<T>>>>,
}

impl<T> InnerListener<T> {
    fn new() -> Self {
        Self {
            trackers: Vec::new(),
        }
    }

    fn add_tracker(&mut self, tracker: Rc<RefCell<InnerTracker<T>>>) {
        self.trackers.push(tracker);
    }

    fn remove_tracker(&mut self, tracker: Rc<RefCell<InnerTracker<T>>>) {
        let found_index = self
            .trackers
            .iter()
            .position(|it| it.borrow().handle() == tracker.borrow().handle());
        found_index.iter().for_each(|&idx| {
            let _ = self.trackers.remove(idx);
        });
    }

    fn emit(&mut self, data: T)
    where
        T: Clone,
    {
        self.trackers
            .iter_mut()
            .for_each(|tracker| tracker.borrow_mut().push(data.clone()));
    }
}

#[cfg(test)]
mod tests {
    // workaround to avoid false positive linter warnings `unused_extern_crates` until
    // Rust issue [#57274](https://github.com/rust-lang/rust/issues/57274) is solved.
    use assertor as _;
}
