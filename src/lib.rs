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
