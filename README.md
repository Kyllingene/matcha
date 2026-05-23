# `matcha`

A framework for breaking down token streams in proc macros.

The macros
[`decompose!`](https://docs.rs/matcha/latest/matcha/macro.decompose.html) and
[`compose!`](https://docs.rs/matcha/latest/matcha/macro.compose.html) are
the main entrypoints for this crate. However, it’s worth familiarizing yourself
with all the types/traits.

## Features

The following features are provided (all are enabled by default).

- `quote`: enables [`quote::ToTokens`](https://docs.rs/quote/1.0.45/quote/to_tokens/trait.ToTokens.html) impls for various types.
- `proc-macro2`: uses the [`proc-macro2`](https://crates.io/crates/proc-macro2) crate instead of the
  built-in `proc-macro`.
- `proc-macro2-span-locations`: enables the `proc-macro2` feature
  `span-locations`, allowing [errors](https://docs.rs/matcha/latest/matcha/stream/struct.Error.html) to print their locations.

## License

This project is licensed under either the MIT or Apache 2.0 license, at your
option. See `LICENSE-MIT` and `LICENSE-APACHE` for details.
