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

#[derive(Debug, Error)]
pub enum IommufdError {
    #[error("failed to open /dev/iommufd: {0}")]
    OpenIommufd(#[source] io::Error),
    #[error("failed to allocate IOAS: {0}")]
    IommuIoasAlloc(#[source] SysError),
    #[error("failed to map an IOVA range to the IOAS: {0}")]
    IommuIoasMap(#[source] SysError),
    #[error("failed to unmap an IOVA range from the IOAS: {0}")]
    IommuIoasUnmap(#[source] SysError),
}

pub type Result<T> = std::result::Result<T, IommufdError>;
