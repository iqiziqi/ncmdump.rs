//! This is a simple way to decrypt the ncm file.
//!
//! # Install
//!
//! You can add this to your `Cargo.toml`:
//!
//! ```toml
//! ncmdump = "0.7.1"
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
//! use std::io::{Error, Write};
//! use std::path::Path;
//!
//! use ncmdump::Ncmdump;
//!
//! fn main() -> Result<(), Error> {
//!     let file = File::open("res/test.ncm")?;
//!     let mut ncm = Ncmdump::from_reader(file).expect("Can't create dump");
//!     let music = ncm.get_data().expect("Can't get data");
//!     let mut target = File::options()
//!         .create(true)
//!         .write(true)
//!         .open("res/test.flac")?;
//!     target.write_all(&music)?;
//!     Ok(())
//! }
//! ```
//!
#[cfg(feature = "ncmdump")]
pub use crate::ncmdump::Ncmdump;
#[cfg(feature = "ncmdump")]
pub use crate::ncmdump::NcmInfo;
#[cfg(feature = "qmcdump")]
pub use crate::qmcdump::QmcDump;

#[cfg(feature = "ncmdump")]
mod ncmdump;
#[cfg(feature = "qmcdump")]
mod qmcdump;

pub mod error;
#[cfg(feature = "utils")]
pub mod utils;
