# iommufd

The `iommufd` workspace hosts libraries related to Rust bindings and
wrappers to the
[IOMMUFD](https://docs.kernel.org/userspace-api/iommufd.html) subsystem
from the Linux kernel. It currently consists of the following crates:

- `iommufd-bindings` -> Rust FFI bindings to IOMMUFD generated using bindgen
- `iommufd-ioctls` -> Safe wrappers over IOMMUFD uAPIs