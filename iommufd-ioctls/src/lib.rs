// Copyright © 2025 Crusoe Energy Systems LLC
//
// SPDX-License-Identifier: Apache-2.0
//

#[macro_use]
extern crate vmm_sys_util;

use std::io;
use thiserror::Error;
use vmm_sys_util::errno::Error as SysError;

pub mod iommufd_ioctls;

pub use iommufd_ioctls::*;

use iommufd_bindings::iommufd::*;

#[derive(Debug, Error)]
pub enum IommufdError {
    #[error("failed to open /dev/iommufd: {0}")]
    OpenIommufd(#[source] io::Error),
    #[error("failed to destroy iommufd object: {0}")]
    IommuDestroy(#[source] SysError),
    #[error("failed to allocate IOAS: {0}")]
    IommuIoasAlloc(#[source] SysError),
    #[error("failed to map an IOVA range to the IOAS: {0}")]
    IommuIoasMap(#[source] SysError),
    #[error("failed to unmap an IOVA range from the IOAS: {0}")]
    IommuIoasUnmap(#[source] SysError),
    #[error("failed to allocate HWPT: {0}")]
    IommuHwptAlloc(#[source] SysError),
    #[error("failed to allocate vIOMMU: {0}")]
    IommuViommuAlloc(#[source] SysError),
    #[error("failed to allocate vDevice: {0}")]
    IommuVdeviceAlloc(#[source] SysError),
    #[error("failed to get HW info: {0}")]
    IommuGetHwInfo(#[source] SysError),
    #[error("failed to invalidate HWPT: {0}")]
    IommuHwptInvalidate(#[source] SysError),
    #[error("failed to allocate vEVENTQ: {0}")]
    IommuVeventqAlloc(#[source] SysError),
    #[error("failed to make the vEVENTQ fd non-blocking: {0}")]
    VeventqNonBlocking(#[source] SysError),
    #[error("failed to allocate HW queue: {0}")]
    IommuHwQueueAlloc(#[source] SysError),
    #[error("hardware queues are not available on this vIOMMU")]
    HwQueueUnsupported,
    #[error("unsupported vIOMMU type: {0}")]
    UnsupportedViommuType(iommu_viommu_type),
    #[error("unsupported vEVENTQ type: {0}")]
    UnsupportedVeventqType(iommu_veventq_type),
    #[error("unsupported IOMMU: {0}")]
    UnsupportedIommu(iommu_hw_info_type),
    #[error("the data does not match the backing IOMMU: {0:?}")]
    IommuTypeMismatch(IommuKind),
    #[error("S1 HWPT already allocated with for the given vDevice: {0}")]
    S1HwptAlreadyAllocated(u32),
    #[error("failed to attach the device to a page table: {0}")]
    AttachHwpt(#[source] io::Error),
}

pub type Result<T> = std::result::Result<T, IommufdError>;
