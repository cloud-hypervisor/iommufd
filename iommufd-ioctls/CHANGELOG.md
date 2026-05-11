## Upcoming release

## Changed

## Added

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
