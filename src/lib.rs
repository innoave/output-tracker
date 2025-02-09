#[cfg(feature = "asynchronous")]
pub mod asynchronous;
mod inner_subject;
mod inner_tracker;
#[cfg(feature = "non-threadsafe")]
pub mod non_threadsafe;
#[cfg(feature = "threadsafe")]
pub mod threadsafe;
mod tracker_handle;

// workaround for false positive 'unused extern crate' warnings until
// Rust issue [#95513](https://github.com/rust-lang/rust/issues/95513) is fixed
#[cfg(test)]
mod tests {
    use thiserror as _;
}
