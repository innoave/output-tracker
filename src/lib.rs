#[cfg(feature = "asynchronous")]
pub mod asynchronous;
mod inner_listener;
mod inner_tracker;
#[cfg(feature = "non-threadsafe")]
pub mod non_threadsafe;
#[cfg(feature = "threadsafe")]
pub mod threadsafe;
mod tracker_handle;

#[cfg(test)]
mod tests {
    // workaround to avoid false positive linter warnings `unused_extern_crates` until
    // Rust issue [#57274](https://github.com/rust-lang/rust/issues/57274) is solved.
    use assertor as _;
}
