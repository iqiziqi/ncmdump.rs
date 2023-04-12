//! This is a simple way to decrypt the ncm file.
//!
//! # Install
//!
//! You can add this to your `Cargo.toml`:
//!
//! ```toml
//! ncmdump = "0.5.0"
//! ```
//!
//! Also, you can use this command to install this crate,
//! if you installed [cargo-edit](https://github.com/killercup/cargo-edit)
//!
//! ```shell
//! cargo add ncmdump
//! ```
//!
//! # Usage
//!
//! ```rust
//! use std::fs::File;
//! use std::io::Write;
//! use std::path::Path;
//!
//! use anyhow::Result;
//! use ncmdump::Ncmdump;
//!
//! fn main() -> Result<()> {
//!
//!     let file = File::open("tests/test.ncm")?;
//!     let mut ncm = Ncmdump::from_reader(file)?;
//!     let music = ncm.get_data()?;
//!     let mut target = File::options()
//!         .create(true)
//!         .write(true)
//!         .open("tests/test.flac")?;
//!     target.write_all(&music)?;
//!     Ok(())
//! }
//! ```
//!
#[cfg(feature = "ncmdump")]
mod ncmdump;
#[cfg(feature = "qmcdump")]
mod qmcdump;

pub mod error;
#[cfg(feature = "utils")]
pub mod utils;

#[cfg(feature = "ncmdump")]
pub use crate::ncmdump::NcmInfo;
#[cfg(feature = "ncmdump")]
pub use crate::ncmdump::Ncmdump;

#[cfg(feature = "qmcdump")]
pub use crate::qmcdump::QmcDump;
