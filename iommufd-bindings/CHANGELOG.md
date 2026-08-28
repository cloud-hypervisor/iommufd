## Upcoming release

## Changed

## Added

## Fixed


# [v0.2.0]

## Changed
- Regenerate the bindings from kernel
  [v7.1](https://github.com/torvalds/linux/tree/v7.1).

## Added
- vIOMMU, vDevice, vEVENTQ and HW queue uAPIs, together with the ARM SMMUv3
  hardware info, invalidation and event types.

## Fixed


# [v0.1.0]

This is the first `iommufd-bindings` crate release.

This crate provides Rust FFI bindings to iommufd uAPIs, generated using
[bindgen](https://crates.io/crates/bindgen). Currently, the bindings are
generated using bindgen version 0.72.0 and kernel version
[v6.6](https://github.com/torvalds/linux/tree/v6.6).
