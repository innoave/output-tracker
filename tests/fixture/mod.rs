// workaround for false positive 'unused extern crate' warnings until
// Rust issue [#95513](https://github.com/rust-lang/rust/issues/95513) is fixed
mod dummy_extern_uses {
    use assertor as _;
    use output_tracker as _;
    use proptest as _;
    use thiserror as _;
    use version_sync as _;
}
