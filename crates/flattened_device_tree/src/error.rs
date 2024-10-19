// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use fdt_rs::error::DevTreeError;
use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum FdtError {
    #[error("FDT pointer not aligned")]
    FdtPointerNotAligned(),
    #[error("FDT error: {0}")]
    FdtErrorParsing(#[from] DevTreeError),
    #[error("No memory node")]
    NoMemoryNode(),
}
