## Upcoming release

## Changed

## Added

## Fixed

# [v0.4.0]

## Changed

- `IommufdVIommu::new()` takes an `Arc<IommufdIoas>` rather than an `IommuFd`
  and an IOAS identifier
- `IommufdVDevice::new()` takes the device it stands for, which
  `install_s1_hwpt()` and `uninstall_s1_hwpt()` no longer take
- Dropping an `IommufdVDevice` now also detaches the device from the page
  table it is bound to, so the caller no longer has to order teardown itself
- `install_s1_hwpt()` now replaces the page table atomically
- Removed `IommufdError::S1HwptAlreadyAllocated`

## Added

- Add IommufdIoas abstraction
- Add IOMMU_IOAS_MAP_FILE support

## Fixed

# [v0.3.1]

## Changed

- Extended `AttachHwpt` trait with method `detach_hwpt()` to allow the page table to be detached from the associated device.

## Added

## Fixed

# [v0.3.0]

## Changed
- Require `iommufd-bindings` 0.2.0.

## Added
- Add `Iommufd` wrappers for the vIOMMU, vDevice, vEVENTQ and HW queue ioctls.
- Add `IommufdVIommu` and `IommufdVDevice`, which own the lifetime of a vIOMMU,
  its stage 2 HWPT and the per device stage 1 HWPTs, and expose invalidation and
  hardware info queries.
- Add `IommufdVEventQ` and ARM SMMUv3 event decoding.
- Add the `AttachHwpt` trait so a caller can attach a device to a stage 1
  HWPT without depending on how the device was opened.
- Add `IommufdHwQueue` for hardware accelerated command queues, enabled with
  the `hw_queue` flag through `IommufdVIommu::new`.

## Fixed

# [v0.2.0]

## Changed

## Added
- [[9]](https://github.com/cloud-hypervisor/iommufd/pull/9) Add `Iommufd::destroy_iommu_object` to release iommufd
  objects explicitly.
- [[9]](https://github.com/cloud-hypervisor/iommufd/pull/9) Add `Iommufd::new_from_fd` to construct a `Iommufd` from
  a pre-opened iommufd file.

## Fixed

# [v0.1.0]

This is the first `iommufd-ioctls` crate release.

The iommufd-ioctls crate provides safe wrappers over the
[IOMMUFD uAPIs](https://docs.kernel.org/userspace-api/iommufd.html#iommufd-user-api), a set
of ioctls used to control the IOMMU subsystem as it relates to managing
IO page tables from userspace. The ioctls are accessible through
structure `IommuFd`.
