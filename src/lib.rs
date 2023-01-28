//! This is a simple way to decrypt the ncm file.
//!
//! # Install
//!
//! You can add this to your `Cargo.toml`:
//!
//! ```toml
//! ncmdump = "0.3.0"
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
mod decrypt;
mod deprecated;
mod ncmdump;
mod qmcdump;
mod utils;

pub mod error;

pub use crate::deprecated::*;
pub use crate::ncmdump::Ncmdump;
pub use crate::qmcdump::QmcDump;
