# Change Log

An overview of changes:

## [0.3.0]
*These notes include a [little upgrade guide below](##Upgrading).*

This release updates `audiopus_sys` to `0.3`, bringing following
changes to this high-level crate:

### **Changed:**
* **Important**: `cmake` is now required to build Opus.

* The API now expects you to provide the already converted structures instead of
accepting a type implementing `TryInto` for the structure.

* Windows will build Opus instead of using a pre-built version.

### **Fixed:**

* Cross-compiling should work now.

### **Removed:**

* Pre-built Windows binaries are no longer provided.

### **Upgrading:**
Arguments for the methods in `audiopus` use newtypes verifying whether
constraints are upheld.

This new update requires the API-user to provide these
structures instead of passing a type that would `TryInto` the structure.
However this can be done by using `TryFrom` or `TryInto`.

Constructing them via `T::try_from`:

```rust
let mut signals = MutSignals::try_from(&mut signals)?;
```

or by converting them inside the method via `value.try_into()?`:

```rust
soft_clip.apply(&(frames).try_into()?)?;
```

Let's look at code on how to use the old `v0.2` and compare it to new `v0.3`

Old `v0.2`:
```rust
    let mut soft_clip = SoftClip::new(Channels::Stereo);

    let mut signals: Vec<f32> = vec![];
    /// The raw data is being processed inside the method.
    soft_clip.apply(signals)?;
```

New `v0.3`:
```rust
    let mut soft_clip = SoftClip::new(Channels::Stereo);

    let mut signals: Vec<f32> = vec![];

    soft_clip.apply((&signals).try_into()?)?;
```
This optimises for compile time – as generics have been eliminated – improves the API clarity, and it allows the
user to create and handle the structure construction errors one-by-one.

## [0.2.0]

This release fixes an API inconsistency by introducing one breaking change.

The `input` on `Decoder::decode_float` must be `Option<TP>`,
to be consistent with the `Decoder::decode`-method.
This allows users to provide a null raw pointer by passing `None`.

## [0.1.3]

The `Decoder` implements `Send` now, to be consistent with `Encoder`.

### **Added:**

* Implement `Send` for `Decoder`.

## [0.1.2]

This release contains a critical fix for `SoftClip::apply`.

### **Fixed:**

* Fixed potential Segfault caused by `SoftClip::apply`.
* A typo.

## [0.1.1]

### **Added:**

* Implements `std::error::Error` for `Error` and `ErrorCode`.


