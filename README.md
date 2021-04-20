[![ci-badge][]][ci] [![docs-badge][]][docs] [![rust 1.33+ badge]][rust 1.33+ link] [![crates.io version]][crates.io link]

# About

`audiopus` is a high-level binding of [`Opus`] version 1.3.
The crate uses [`audiopus_sys`] underneath.

Orginally, this crate was made to empower the [`serenity`]-crate to build audio features on Windows, Linux, and Mac.

Everyone is welcome to contribute,
check out the [`CONTRIBUTING.md`](CONTRIBUTING.md) for further guidance.

# Building

## Requirements
If you want to build Opus, you will need `cmake`.

If you have `pkg-config`, it will attempt to use that before building.

You can also link a pre-installed Opus, see [**Pre-installed Opus**](#Pre-installed-Opus)
below.

This crate provides a pre-built binding. In case you want to generate the
binding yourself, you will need [`Clang`](https://rust-lang.github.io/rust-bindgen/requirements.html#clang),
see [**Pre-installed Opus**](#Generating-The-Binding) below for further
instructions.

## Linking
The crate underneath, [`audiopus_sys`], links to Opus 1.3 and supports Windows, Linux, and MacOS
By default, we statically link to Windows, MacOS, and if you use the
`musl`-environment. We will link dynamically for Linux except when using
mentioned `musl`.

This can be altered by compiling with the `static` or `dynamic` feature having
effects respective to their names. If both features are enabled,
we will pick your system's default.

Environment variables named `LIBOPUS_STATIC` or `OPUS_STATIC` will take
precedence over features thus overriding the behaviour. The value of these
environment variables have no influence of the result: If one of them is set,
statically linking will be picked.

## Pkg-Config
By default, `audiopus_sys` will use `pkg-config` on Unix or GNU.
Setting the environment variable `LIBOPUS_NO_PKG` or `OPUS_NO_PKG` will bypass
probing for Opus via `pkg-config`.

## Pre-installed Opus
If you have Opus pre-installed, you can set `LIBOPUS_LIB_DIR` or
`OPUS_LIB_DIR` to the directory containing Opus.

Be aware that using an Opus other than version 1.3 may not work.

# Installation
Add this to your `Cargo.toml`:

```toml
[dependencies]
audiopus = "0.3"
```
[`serenity`]: https://crates.io/crates/serenity

[`Opus`]: https://www.opus-codec.org/

[`audiopus_sys`]: https://github.com/Lakelezz/audiopus_sys.git

[ci]: https://github.com/Lakelezz/audiopus/actions
[ci-badge]: https://img.shields.io/github/workflow/status/Lakelezz/audiopus/CI?style=flat-square

[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg?style=flat-square&colorB=32b6b7
[docs]: https://docs.rs/audiopus

[rust 1.33+ badge]: https://img.shields.io/badge/rust-1.33+-93450a.svg?style=flat-square&colorB=ff9a0d
[rust 1.33+ link]: https://blog.rust-lang.org/2019/02/28/Rust-1.33.0.html

[crates.io link]: https://crates.io/crates/audiopus
[crates.io version]: https://img.shields.io/crates/v/audiopus.svg?style=flat-square&colorB=b73732
