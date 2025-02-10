#![doc(html_root_url = "https://docs.rs/output-tracker/0.1.0")]

mod inner_subject;
mod inner_tracker;
#[cfg(any(feature = "non-threadsafe", not(feature = "threadsafe")))]
pub mod non_threadsafe;
#[cfg(feature = "threadsafe")]
pub mod threadsafe;
mod tracker_handle;

// test code snippets in the README.md
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
#[allow(dead_code)]
type TestExamplesInReadme = ();

// workaround for false positive 'unused extern crate' warnings until
// Rust issue [#95513](https://github.com/rust-lang/rust/issues/95513) is fixed
#[cfg(test)]
mod dummy_extern_uses {
    use version_sync as _;
}
