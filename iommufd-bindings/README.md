# iommufd-bindings
Rust FFI bindings to iommufd uAPIs, generated using
[bindgen](https://crates.io/crates/bindgen). The bindings exported by
this crate are statically generated using header files associated with
a specific kernel version, and are not automatically synced with the
kernel version running on a particular host. The user must ensure that
specific structures, members, or constants are supported and valid for the
kernel version they are using.

Currently, the bindings are generated using bindgen version 0.72.0 and
kernel version [v6.6](https://github.com/torvalds/linux/tree/v6.6).

## Regenerating Bindings

### Bindgen
Install bindgen version 0.72.0
```bash
cargo install bindgen-cli --vers 0.72.0
```

### Linux Kernel
Generating bindings depends on the Linux kernel, so you need to have the
repository on your machine.

```bash
git clone https://github.com/torvalds/linux.git
```

### Example for regenerating

For this example we assume that you have both linux and iommufd-bindings
repositories in your root and we use linux version v6.6 as example.

```bash
# linux is the repository that you cloned previously.
cd linux

# Step 1: Checkout the version you want to generate the bindings for.
git checkout tags/v6.6

# Step 2: Generate the bindings from the kernel headers.
make headers_install INSTALL_HDR_PATH=iommufd_headers
cd iommufd_headers
bindgen include/linux/iommufd.h -o iommufd.rs \
    --impl-debug --with-derive-default  \
    --with-derive-partialeq  --impl-partialeq \
    -- -Iinclude

cd ~

# Step 3: Copy the generated files to the new version module.
cp linux/iommufd_headers/iommufd.rs iommufd-bindings/src/iommufd.rs
```