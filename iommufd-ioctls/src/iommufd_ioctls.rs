// Copyright © 2025 Crusoe Energy Systems LLC
//
// SPDX-License-Identifier: Apache-2.0
//

use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::Arc;

use iommufd_bindings::iommufd::*;
use vmm_sys_util::errno::Error as SysError;

use crate::{IommufdError, Result};

pub struct IommuFd {
    iommufd: File,
}

impl IommuFd {
    pub fn new() -> Result<Self> {
        let iommufd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/iommu")
            .map_err(IommufdError::OpenIommufd)?;

        Ok(IommuFd { iommufd })
    }

    /// New from an already-opened `/dev/iommu` file. Caller transfers
    /// ownership; closing the fd is now `IommuFd`'s responsibility.
    pub fn new_from_fd(file: File) -> Self {
        IommuFd { iommufd: file }
    }

    pub fn destroy_iommu_object(&self, id: u32) -> Result<()> {
        let destroy_data = iommu_destroy {
            size: std::mem::size_of::<iommu_destroy>() as u32,
            id,
        };
        iommufd_syscall::destroy_iommu_object(self, &destroy_data)
    }

    pub fn alloc_iommu_ioas(&self, alloc_data: &mut iommu_ioas_alloc) -> Result<()> {
        iommufd_syscall::alloc_iommu_ioas(self, alloc_data)
    }

    pub fn map_iommu_ioas(&self, map: &iommu_ioas_map) -> Result<()> {
        iommufd_syscall::map_iommu_ioas(self, map)
    }

    pub fn unmap_iommu_ioas(&self, unmap: &mut iommu_ioas_unmap) -> Result<()> {
        iommufd_syscall::unmap_iommu_ioas(self, unmap)
    }

    pub fn alloc_iommu_hwpt(&self, hwpt_alloc: &mut iommu_hwpt_alloc) -> Result<()> {
        iommufd_syscall::alloc_iommu_hwpt(self, hwpt_alloc)
    }

    pub fn get_hw_info(&self, hw_info: &mut iommu_hw_info) -> Result<()> {
        iommufd_syscall::get_hw_info(self, hw_info)
    }

    pub fn invalidate_hwpt(&self, hwpt_invalidate: &mut iommu_hwpt_invalidate) -> Result<()> {
        iommufd_syscall::invalidate_hwpt(self, hwpt_invalidate)
    }

    pub fn alloc_iommu_viommu(&self, viommu_alloc: &mut iommu_viommu_alloc) -> Result<()> {
        iommufd_syscall::alloc_iommu_viommu(self, viommu_alloc)
    }

    pub fn alloc_iommu_vdevice(&self, vdevice_alloc: &mut iommu_vdevice_alloc) -> Result<()> {
        iommufd_syscall::alloc_iommu_vdevice(self, vdevice_alloc)
    }

    pub fn device_hw_info(
        &self,
        dev_id: u32,
        hw_info_data: &mut IommufdHwInfoData,
    ) -> Result<iommu_hw_info> {
        let mut hw_info = match hw_info_data {
            IommufdHwInfoData::Smmuv3(data) => iommu_hw_info {
                size: std::mem::size_of::<iommu_hw_info>() as u32,
                dev_id,
                data_len: std::mem::size_of::<iommu_hw_info_arm_smmuv3>() as u32,
                data_uptr: data as *mut _ as u64,
                ..Default::default()
            },
        };

        self.get_hw_info(&mut hw_info)?;

        Ok(hw_info)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum IommufdInvalidateData {
    Smmuv3(iommu_viommu_arm_smmuv3_invalidate),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IommuKind {
    Smmuv3,
    Smmuv3Cmdqv,
    Unsupported(iommu_hw_info_type),
}

impl IommuKind {
    fn reports(iommufd: &IommuFd, dev_id: u32, hw_info_type: iommu_hw_info_type) -> bool {
        let mut hw_info = iommu_hw_info {
            size: std::mem::size_of::<iommu_hw_info>() as u32,
            flags: iommufd_hw_info_flags_IOMMU_HW_INFO_FLAG_INPUT_TYPE,
            dev_id,
            __bindgen_anon_1: iommu_hw_info__bindgen_ty_1 {
                in_data_type: hw_info_type,
            },
            ..Default::default()
        };

        iommufd.get_hw_info(&mut hw_info).is_ok()
    }

    #[allow(non_upper_case_globals)]
    fn query(iommufd: &IommuFd, dev_id: u32) -> Result<Self> {
        let mut hw_info = iommu_hw_info {
            size: std::mem::size_of::<iommu_hw_info>() as u32,
            dev_id,
            ..Default::default()
        };
        iommufd.get_hw_info(&mut hw_info)?;

        // SAFETY: the union holds one `__u32` under either name.
        let hw_info_type = unsafe { hw_info.__bindgen_anon_1.out_data_type };

        match hw_info_type {
            iommu_hw_info_type_IOMMU_HW_INFO_TYPE_TEGRA241_CMDQV => Ok(IommuKind::Smmuv3Cmdqv),
            iommu_hw_info_type_IOMMU_HW_INFO_TYPE_ARM_SMMUV3 => {
                if Self::reports(
                    iommufd,
                    dev_id,
                    iommu_hw_info_type_IOMMU_HW_INFO_TYPE_TEGRA241_CMDQV,
                ) {
                    Ok(IommuKind::Smmuv3Cmdqv)
                } else {
                    Ok(IommuKind::Smmuv3)
                }
            }
            _ => Ok(IommuKind::Unsupported(hw_info_type)),
        }
    }
}

pub struct IommufdVIommu {
    iommufd: Arc<IommuFd>,
    viommu_id: u32,
    s2_hwpt_id: u32,
    bypass_hwpt_id: Option<u32>,
    abort_hwpt_id: Option<u32>,
    kind: IommuKind,
}

impl IommufdVIommu {
    pub fn new(iommufd: Arc<IommuFd>, ioas_id: u32, dev_id: u32) -> Result<Self> {
        let kind = IommuKind::query(&iommufd, dev_id)?;

        let (bypass_s1_hwpt_data, abort_s1_hwpt_data) = match kind {
            IommuKind::Smmuv3 | IommuKind::Smmuv3Cmdqv => {
                const SMMU_STE_VALID: u64 = 1 << 0;
                const SMMU_STE_CFG_BYPASS: u64 = 1 << 3;

                (
                    IommufdHwptData::Smmuv3(iommu_hwpt_arm_smmuv3 {
                        ste: [SMMU_STE_CFG_BYPASS | SMMU_STE_VALID, 0x0],
                    }),
                    IommufdHwptData::Smmuv3(iommu_hwpt_arm_smmuv3 {
                        ste: [SMMU_STE_VALID, 0x0],
                    }),
                )
            }
            IommuKind::Unsupported(hw_info_type) => {
                return Err(IommufdError::UnsupportedIommu(hw_info_type));
            }
        };

        let mut s2_iommufd_hwpt_alloc = iommu_hwpt_alloc {
            size: std::mem::size_of::<iommu_hwpt_alloc>() as u32,
            flags: iommufd_hwpt_alloc_flags_IOMMU_HWPT_ALLOC_NEST_PARENT,
            dev_id,
            pt_id: ioas_id,
            data_type: iommu_hwpt_data_type_IOMMU_HWPT_DATA_NONE,
            ..Default::default()
        };
        iommufd.alloc_iommu_hwpt(&mut s2_iommufd_hwpt_alloc)?;
        let s2_hwpt_id = s2_iommufd_hwpt_alloc.out_hwpt_id;

        let mut viommu_alloc = iommu_viommu_alloc {
            size: std::mem::size_of::<iommu_viommu_alloc>() as u32,
            type_: iommu_viommu_type_IOMMU_VIOMMU_TYPE_ARM_SMMUV3,
            hwpt_id: s2_hwpt_id,
            dev_id,
            ..Default::default()
        };
        iommufd.alloc_iommu_viommu(&mut viommu_alloc)?;
        let viommu_id = viommu_alloc.out_viommu_id;

        let mut viommu = IommufdVIommu {
            iommufd,
            viommu_id,
            s2_hwpt_id,
            bypass_hwpt_id: None,
            abort_hwpt_id: None,
            kind,
        };
        viommu.bypass_hwpt_id = Some(viommu.alloc_s1_hwpt(dev_id, &bypass_s1_hwpt_data)?);
        viommu.abort_hwpt_id = Some(viommu.alloc_s1_hwpt(dev_id, &abort_s1_hwpt_data)?);

        Ok(viommu)
    }

    pub fn alloc_s1_hwpt(&self, dev_id: u32, hwpt_data: &IommufdHwptData) -> Result<u32> {
        let (data_type, data_len, data_uptr) = match (self.kind, hwpt_data) {
            (IommuKind::Smmuv3 | IommuKind::Smmuv3Cmdqv, IommufdHwptData::Smmuv3(data)) => (
                iommu_hwpt_data_type_IOMMU_HWPT_DATA_ARM_SMMUV3,
                std::mem::size_of::<iommu_hwpt_arm_smmuv3>() as u32,
                data as *const iommu_hwpt_arm_smmuv3 as u64,
            ),
            (kind, _) => return Err(IommufdError::IommuTypeMismatch(kind)),
        };

        let mut hwpt_alloc = iommu_hwpt_alloc {
            size: std::mem::size_of::<iommu_hwpt_alloc>() as u32,
            dev_id,
            pt_id: self.viommu_id,
            data_type,
            data_len,
            data_uptr,
            ..Default::default()
        };
        self.iommufd.alloc_iommu_hwpt(&mut hwpt_alloc)?;

        Ok(hwpt_alloc.out_hwpt_id)
    }

    pub fn invalidate(&self, cmd: &mut IommufdInvalidateData) -> Result<bool> {
        match (self.kind, cmd) {
            (IommuKind::Smmuv3 | IommuKind::Smmuv3Cmdqv, IommufdInvalidateData::Smmuv3(data)) => {
                let mut hw_invalidate = iommu_hwpt_invalidate {
                    size: std::mem::size_of::<iommu_hwpt_invalidate>() as u32,
                    hwpt_id: self.viommu_id,
                    data_type:
                        iommu_hwpt_invalidate_data_type_IOMMU_VIOMMU_INVALIDATE_DATA_ARM_SMMUV3,
                    entry_len: std::mem::size_of::<iommu_viommu_arm_smmuv3_invalidate>() as u32,
                    entry_num: 1,
                    data_uptr: data as *mut iommu_viommu_arm_smmuv3_invalidate as u64,
                    ..Default::default()
                };
                self.iommufd.invalidate_hwpt(&mut hw_invalidate)?;

                if hw_invalidate.entry_num == 1 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            (kind, _) => Err(IommufdError::IommuTypeMismatch(kind)),
        }
    }
}

impl Drop for IommufdVIommu {
    fn drop(&mut self) {
        for (id, what) in [
            (self.bypass_hwpt_id, "bypass_hwpt"),
            (self.abort_hwpt_id, "abort_hwpt"),
            (Some(self.viommu_id), "vIOMMU"),
            (Some(self.s2_hwpt_id), "s2_hwpt"),
        ] {
            let Some(id) = id else { continue };
            if let Err(e) = self.iommufd.destroy_iommu_object(id) {
                eprintln!("Failed to destroy {what} id {id}: {e}");
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum IommufdHwInfoData {
    Smmuv3(iommu_hw_info_arm_smmuv3),
}

#[derive(Debug, Copy, Clone)]
pub enum IommufdHwptData {
    Smmuv3(iommu_hwpt_arm_smmuv3),
}

pub struct IommufdVDevice {
    viommu: Arc<IommufdVIommu>,
    dev_id: u32,
    virt_id: u64,
    vdevice_id: u32,
    s1_hwpt_id: Option<u32>,
}

impl IommufdVDevice {
    pub fn new(viommu: Arc<IommufdVIommu>, dev_id: u32, virt_id: u64) -> Result<Self> {
        let mut vdevice_alloc = iommu_vdevice_alloc {
            size: std::mem::size_of::<iommu_vdevice_alloc>() as u32,
            viommu_id: viommu.viommu_id,
            dev_id,
            virt_id,
            ..Default::default()
        };
        viommu.iommufd.alloc_iommu_vdevice(&mut vdevice_alloc)?;

        Ok(IommufdVDevice {
            viommu,
            dev_id,
            virt_id,
            vdevice_id: vdevice_alloc.out_vdevice_id,
            s1_hwpt_id: None,
        })
    }

    pub fn allocate_s1_hwpt(&mut self, hwpt_data: &IommufdHwptData) -> Result<u32> {
        if self.s1_hwpt_id.is_some() {
            return Err(IommufdError::S1HwptAlreadyAllocated(self.vdevice_id));
        }

        let s1_hwpt_id = self.viommu.alloc_s1_hwpt(self.dev_id, hwpt_data)?;
        self.s1_hwpt_id = Some(s1_hwpt_id);

        Ok(s1_hwpt_id)
    }

    pub fn destroy_s1_hwpt(&mut self) -> Result<()> {
        if let Some(s1_hwpt_id) = self.s1_hwpt_id {
            self.viommu.iommufd.destroy_iommu_object(s1_hwpt_id)?;
            self.s1_hwpt_id = None;
        }
        Ok(())
    }

    pub fn hw_info(&self, hw_info_data: &mut IommufdHwInfoData) -> Result<iommu_hw_info> {
        match (self.viommu.kind, &hw_info_data) {
            (IommuKind::Smmuv3 | IommuKind::Smmuv3Cmdqv, IommufdHwInfoData::Smmuv3(_)) => {}
            (kind, _) => return Err(IommufdError::IommuTypeMismatch(kind)),
        }

        self.viommu
            .iommufd
            .device_hw_info(self.dev_id, hw_info_data)
    }

    pub fn virt_id(&self) -> u64 {
        self.virt_id
    }
}

impl Drop for IommufdVDevice {
    fn drop(&mut self) {
        if let Some(s1_hwpt_id) = self.s1_hwpt_id
            && let Err(e) = self.viommu.iommufd.destroy_iommu_object(s1_hwpt_id)
        {
            eprintln!("Failed to destroy s1_hwpt id {s1_hwpt_id}: {e}");
        }

        if let Err(e) = self.viommu.iommufd.destroy_iommu_object(self.vdevice_id) {
            eprintln!("Failed to destroy vDevice id {}: {e}", self.vdevice_id);
        }
    }
}

impl AsRawFd for IommuFd {
    fn as_raw_fd(&self) -> RawFd {
        self.iommufd.as_raw_fd()
    }
}

ioctl_io_nr!(IOMMU_DESTROY, IOMMUFD_TYPE as u32, IOMMUFD_CMD_DESTROY);
ioctl_io_nr!(
    IOMMU_IOAS_ALLOC,
    IOMMUFD_TYPE as u32,
    IOMMUFD_CMD_IOAS_ALLOC
);
ioctl_io_nr!(IOMMU_IOAS_MAP, IOMMUFD_TYPE as u32, IOMMUFD_CMD_IOAS_MAP);
ioctl_io_nr!(
    IOMMU_IOAS_UNMAP,
    IOMMUFD_TYPE as u32,
    IOMMUFD_CMD_IOAS_UNMAP
);
ioctl_io_nr!(
    IOMMU_HWPT_ALLOC,
    IOMMUFD_TYPE as u32,
    IOMMUFD_CMD_HWPT_ALLOC
);
ioctl_io_nr!(
    IOMMUFD_GET_HW_INFO,
    IOMMUFD_TYPE as u32,
    IOMMUFD_CMD_GET_HW_INFO
);
ioctl_io_nr!(
    IOMMUFD_HWPT_INVALIDATE,
    IOMMUFD_TYPE as u32,
    IOMMUFD_CMD_HWPT_INVALIDATE
);
ioctl_io_nr!(
    IOMMU_VIOMMU_ALLOC,
    IOMMUFD_TYPE as u32,
    IOMMUFD_CMD_VIOMMU_ALLOC
);
ioctl_io_nr!(
    IOMMU_VDEVICE_ALLOC,
    IOMMUFD_TYPE as u32,
    IOMMUFD_CMD_VDEVICE_ALLOC
);

// Safety:
// - absolutely trust the underlying kernel
// - absolutely trust data returned by the underlying kernel
// - assume kernel will return error if caller passes in invalid file handle, parameter or buffer.
pub(crate) mod iommufd_syscall {
    use super::*;
    use vmm_sys_util::ioctl::{ioctl_with_mut_ref, ioctl_with_ref};

    pub(crate) fn destroy_iommu_object(
        iommufd: &IommuFd,
        destroy_data: &iommu_destroy,
    ) -> Result<()> {
        // SAFETY:
        // 1. The file descriptor provided by 'iommufd' is valid and open.
        // 2. The 'destroy_data' points to initialized memory with expected data structure,
        // and remains valid for the duration of sysca
        // 3. The return value is checked.
        let ret = unsafe { ioctl_with_ref(iommufd, IOMMU_DESTROY(), destroy_data) };
        if ret < 0 {
            Err(IommufdError::IommuDestroy(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn alloc_iommu_ioas(
        iommufd: &IommuFd,
        alloc_data: &mut iommu_ioas_alloc,
    ) -> Result<()> {
        // SAFETY:
        // 1. The file descriptor provided by 'iommufd' is valid and open.
        // 2. The 'alloc_data' points to initialized memory with expected data structure,
        // and remains valid for the duration of syscall.
        // 3. The return value is checked.
        let ret = unsafe { ioctl_with_mut_ref(iommufd, IOMMU_IOAS_ALLOC(), alloc_data) };
        if ret < 0 {
            Err(IommufdError::IommuIoasAlloc(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn map_iommu_ioas(iommufd: &IommuFd, map: &iommu_ioas_map) -> Result<()> {
        // SAFETY:
        // 1. The file descriptor provided by 'iommufd' is valid and open.
        // 2. The 'map' points to initialized memory with expected data structure,
        // and remains valid for the duration of syscall.
        // 3. The return value is checked.
        let ret = unsafe { ioctl_with_ref(iommufd, IOMMU_IOAS_MAP(), map) };
        if ret < 0 {
            Err(IommufdError::IommuIoasMap(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn unmap_iommu_ioas(iommufd: &IommuFd, unmap: &mut iommu_ioas_unmap) -> Result<()> {
        // SAFETY:
        // 1. The file descriptor provided by 'iommufd' is valid and open.
        // 2. The 'unmap' points to initialized memory with expected data structure,
        // and remains valid for the duration of syscall.
        // 3. The return value is checked.
        let ret = unsafe { ioctl_with_mut_ref(iommufd, IOMMU_IOAS_UNMAP(), unmap) };
        if ret < 0 {
            Err(IommufdError::IommuIoasUnmap(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn alloc_iommu_hwpt(
        iommufd: &IommuFd,
        hwpt_alloc: &mut iommu_hwpt_alloc,
    ) -> Result<()> {
        // SAFETY:
        // 1. The file descriptor provided by 'iommufd' is valid and open.
        // 2. The 'hwpt_alloc' points to initialized memory with expected data structure,
        // and remains valid for the duration of syscall.
        // 3. The return value is checked.
        let ret = unsafe { ioctl_with_mut_ref(iommufd, IOMMU_HWPT_ALLOC(), hwpt_alloc) };
        if ret < 0 {
            Err(IommufdError::IommuHwptAlloc(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_hw_info(iommufd: &IommuFd, hw_info: &mut iommu_hw_info) -> Result<()> {
        // SAFETY:
        // 1. The file descriptor provided by 'iommufd' is valid and open.
        // 2. The 'hw_info' points to initialized memory with expected data structure,
        // and remains valid for the duration of syscall.
        // 3. The return value is checked.
        let ret = unsafe { ioctl_with_mut_ref(iommufd, IOMMUFD_GET_HW_INFO(), hw_info) };
        if ret < 0 {
            Err(IommufdError::IommuGetHwInfo(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn invalidate_hwpt(
        iommufd: &IommuFd,
        hwpt_invalidate: &mut iommu_hwpt_invalidate,
    ) -> Result<()> {
        // SAFETY:
        // 1. The file descriptor provided by 'iommufd' is valid and open.
        // 2. The 'hwpt_invalidate' points to initialized memory with expected data structure,
        // and remains valid for the duration of syscall.
        // 3. The return value is checked.
        let ret =
            unsafe { ioctl_with_mut_ref(iommufd, IOMMUFD_HWPT_INVALIDATE(), hwpt_invalidate) };
        if ret < 0 {
            Err(IommufdError::IommuHwptInvalidate(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn alloc_iommu_viommu(
        iommufd: &IommuFd,
        viommu_alloc: &mut iommu_viommu_alloc,
    ) -> Result<()> {
        // SAFETY:
        // 1. The file descriptor provided by 'iommufd' is valid and open.
        // 2. The 'viommu_alloc' points to initialized memory with expected data structure,
        // and remains valid for the duration of syscall.
        // 3. The return value is checked.
        let ret = unsafe { ioctl_with_mut_ref(iommufd, IOMMU_VIOMMU_ALLOC(), viommu_alloc) };
        if ret < 0 {
            Err(IommufdError::IommuViommuAlloc(SysError::last()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn alloc_iommu_vdevice(
        iommufd: &IommuFd,
        vdevice_alloc: &mut iommu_vdevice_alloc,
    ) -> Result<()> {
        // SAFETY:
        // 1. The file descriptor provided by 'iommufd' is valid and open.
        // 2. The 'vdevice_alloc' points to initialized memory with expected data structure,
        // and remains valid for the duration of syscall.
        // 3. The return value is checked.
        let ret = unsafe { ioctl_with_mut_ref(iommufd, IOMMU_VDEVICE_ALLOC(), vdevice_alloc) };
        if ret < 0 {
            Err(IommufdError::IommuVdeviceAlloc(SysError::last()))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iommufd_ioctl_code() {
        assert_eq!(IOMMU_DESTROY(), 15232);
        assert_eq!(IOMMU_IOAS_ALLOC(), 15233);
        assert_eq!(IOMMU_IOAS_MAP(), 15237);
        assert_eq!(IOMMU_IOAS_UNMAP(), 15238);
        assert_eq!(IOMMU_HWPT_ALLOC(), 15241);
        assert_eq!(IOMMUFD_GET_HW_INFO(), 15242);
        assert_eq!(IOMMUFD_HWPT_INVALIDATE(), 15245);
        assert_eq!(IOMMU_VIOMMU_ALLOC(), 15248);
        assert_eq!(IOMMU_VDEVICE_ALLOC(), 15249);
    }
}
