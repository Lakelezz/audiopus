[![ci-badge][]][ci] [![docs-badge][]][docs] [![rust 1.33+ badge]][rust 1.33+ link] [![crates.io version]][crates.io link]

# About

`audiopus` is a high-level binding of [`Opus`] version 1.3.

Orginally, this crate was made to empower the [`serenity`]-crate to build audio features on Windows, Linux, and Mac.

Everyone is welcome to contribute,
check out the [`CONTRIBUTING.md`](CONTRIBUTING.md) for further guidance.

# Building

## Requirements

### UNIX/GNU/MSYS2
You will need `gcc`, `libclang`, `make`, `automake`, `autoconf`, and
`libtool`.
Note that `automake` uses `autoconf` as dependency already.
If you have `pkg-config`, the underlying [`audiopus_sys`]-crate will try finding
Opus with `pkg-config`.

### MSVC
Currently, `audiopus` links to a prebuilt Opus hence should just work.
It supports x86 and x64 as dynamic or static build.

## How Audiopus links
`audiopus` uses [`audiopus_sys`], it links to Opus 1.3 and supports Windows,
Linux, and MacOS.
By default, it statically links to Windows, MacOS, and if you use the
`musl`-environment. It will link dynamically for Linux except when using
mentioned `musl`.

Environment variables named `LIBOPUS_STATIC` or `OPUS_STATIC` will take
precedence over features thus overriding the behaviour. The value of these
environment variables have no influence on the result: If one of them is set,
statically linking will be picked.

## Pre-installed Opus
If you have Opus pre-installed, you can set `LIBOPUS_LIB_DIR` or
`OPUS_LIB_DIR` to point to the directory in which your Opus lies.
Be aware that using an Opus other than version 1.3 may not work.

# Installation
Add this to your `Cargo.toml`:

```toml
[dependencies]
audiopus = "0.1.0"
```
[`serenity`]: https://crates.io/crates/serenity

[`Opus`]: https://www.opus-codec.org/

[`audiopus_sys`]: https://github.com/Lakelezz/audiopus_sys.git

[ci]: https://dev.azure.com/lakeware/audiopus/_build?definitionId=6
[ci-badge]: https://img.shields.io/azure-devops/build/lakeware/9c5a7495-9549-45b5-9f18-a01634be27e2/6/master.svg?style=flat-square

[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg?style=flat-square&colorB=32b6b7
[docs]: https://docs.rs/audiopus

[rust 1.33+ badge]: https://img.shields.io/badge/rust-1.33+-93450a.svg?style=flat-square&colorB=ff9a0d
[rust 1.33+ link]: https://blog.rust-lang.org/2019/02/28/Rust-1.33.0.html

[crates.io link]: https://crates.io/crates/audiopus
[crates.io version]: https://img.shields.io/crates/v/audiopus.svg?style=flat-square&colorB=b73732
