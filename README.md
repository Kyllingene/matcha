# `matcha`

<!-- cargo-reedme: start -->

<!-- cargo-reedme: info-start

    Do not edit this region by hand
    ===============================

    This region was generated from Rust documentation comments by `cargo-reedme` using this command:

        cargo +nightly reedme

    for more info: https://github.com/nik-rev/cargo-reedme

cargo-reedme: info-end -->

A framework for breaking down token streams in proc macros.

The macro [`decompose!`](https://docs.rs/matcha/latest/matcha/macro.decompose.html) is the main entrypoint for this crate. However, the
surrounding types/traits can be useful regardless.

## Features

The following features are provided (all are enabled by default).

- `quote`: enables [`ToTokens`](https://docs.rs/quote/1.0.45/quote/to_tokens/trait.ToTokens.html) impls for various types.
- `proc-macro2`: uses the [`proc-macro2`](proc-macro2) crate instead of the
  built-in `proc-macro`.
- `proc-macro2-span-locations`: enables the `proc-macro2` feature
  `span-locations`, allowing [errors](https://docs.rs/matcha/latest/matcha/stream/struct.Error.html) to print their locations.

<!-- cargo-reedme: end -->

## License

This project is licensed under the MIT license. See `LICENSE.md` for details.
