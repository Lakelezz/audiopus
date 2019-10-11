# Change Log

An overview of changes:

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


