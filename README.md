# rx-rust
Port of Rx for Rust
This is very work-in-progress, and probably shouldn't be used for anything yet.

## Intention
I wanted to learn Rust, and I also had a notion that Rust's affine types may be able to describe a lot of the laws of Rx very well.
Part of my goal is to have a lot of information statically available, which will hopefully allow for better code generation. Right now I think my interfaces might not allow virtualisation very well.

## Future directions

* Allow support for virtualisation, which will probably require wrapping an abstract stream somehow
* Figure out how to represent abnormal stream termination, possibly by parameterising over an additional error type
* Figure out how to handle schedulers
