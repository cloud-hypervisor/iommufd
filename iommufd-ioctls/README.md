# iommufd-ioctls

The iommufd-ioctls crate provides safe wrappers over the
[IOMMUFD uAPIs](https://docs.kernel.org/userspace-api/iommufd.html#iommufd-user-api), a set
of ioctls used to control the IOMMU subsystem as it relates to managing
IO page tables from userspace. The ioctls are accessible through
structure `IommuFd`.
