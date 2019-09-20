//! This is a simple way to decrypt the ncm file.
//!
//! # Install
//!
//! You can add this to your `Cargo.toml`:
//!
//! ```toml
//! ncmdump = 0.1.0
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
//! ```
//! extern crate ncmdump;
//!
//! use std::error::Error;
//! use std::fs::{read, write};
//! use std::path::Path;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let input_path = Path::new("tests/test.ncm");
//!     let output_path = Path::new("tests/test.flac");
//!     let buffer = read(&input_path)?;
//!     let data = ncmdump::decode(&buffer)?;
//!     write(&output_path, data)?;
//!     Ok(())
//! }
//! ```
//!

extern crate aes_soft;
extern crate base64;
extern crate block_modes;
extern crate serde;

mod decrypt;
mod utils;

pub mod error;

use serde::{Deserialize, Serialize};

use crate::decrypt::{build_key_box, decrypt, HEADER_KEY, MODIFY_KEY};
use crate::error::{Error, ErrorKind};
use crate::utils::{check_format, get_length, get_n_element};

#[derive(Debug, Deserialize, Serialize)]
pub struct Modify {
    /// The name of music
    #[serde(rename = "musicName")]
    pub name: String,
    /// The id of music
    #[serde(rename = "musicId")]
    pub id: u64,
    /// The album of music, it's a url
    pub album: String,
    /// The artist of music, first item is name, second item is id
    pub artist: Vec<(String, u64)>,
    // The bit rate of music
    pub bitrate: u64,
    /// The duration of music
    pub duration: u64,
    /// The format of music, is may be 'mp3' or 'flac'
    pub format: String,
    /// The id of MV
    #[serde(rename = "mvId")]
    pub mv_id: Option<u64>,
    /// The alias of music
    pub alias: Option<Vec<String>>,
}

#[derive(Debug)]
struct BlockInfo {
    pub key: Vec<u8>,
    pub modify: Vec<u8>,
    pub crc: Vec<u8>,
    pub image: Vec<u8>,
    pub data: Vec<u8>,
}

fn get_blocks(file_buffer: &[u8]) -> Result<BlockInfo, Error> {
    let mut iter = file_buffer.iter();
    // format area
    {
        let buffer = get_n_element(&mut iter, 10).unwrap();
        check_format(&buffer)?;
    };
    // key area
    let key = {
        let buffer_length = get_n_element(&mut iter, 4).unwrap();
        let length = get_length(&buffer_length)?;
        get_n_element(&mut iter, length as usize).unwrap()
    };
    // modify area
    let modify = {
        let buffer_length = get_n_element(&mut iter, 4).unwrap();
        let length = get_length(&buffer_length)?;
        get_n_element(&mut iter, length as usize).unwrap()
    };
    // crc area
    let crc = get_n_element(&mut iter, 9).unwrap();
    // image area
    let image = {
        let buffer_length = get_n_element(&mut iter, 4).unwrap();
        let length = get_length(&buffer_length)?;
        get_n_element(&mut iter, length as usize).unwrap()
    };
    // data area
    let data: Vec<u8> = iter.as_slice().to_vec();

    Ok(BlockInfo {
        key,
        modify,
        crc,
        image,
        data,
    })
}

fn get_data(key: &[u8], data: &[u8]) -> Vec<u8> {
    let key_box = build_key_box(&key);
    data.chunks(0x8000)
        .map(|i| {
            i.iter().enumerate().map(|(index, item)| {
                let j = (index + 1) & 0xff;
                item ^ key_box[(key_box[j] + key_box[(key_box[j] + j) & 0xff]) & 0xff] as u8
            })
        })
        .flatten()
        .collect::<Vec<u8>>()
}

fn get_key(buffer: &[u8]) -> Result<Vec<u8>, Error> {
    let key_buffer = buffer.iter().map(|byte| byte ^ 0x64).collect::<Vec<u8>>();
    let decrypt_buffer = decrypt(&key_buffer, &HEADER_KEY)?;
    Ok(decrypt_buffer[17..].to_vec())
}

fn get_modify(buffer: &[u8]) -> Result<Modify, Error> {
    let modify_tmp = buffer.iter().map(|item| item ^ 0x63).collect::<Vec<u8>>();
    let modify_key =
        base64::decode(&modify_tmp[22..]).map_err(|_| Error::from(ErrorKind::InvalidFile))?;
    let modify_str = String::from_utf8(decrypt(&modify_key, &MODIFY_KEY)?[6..].to_vec())
        .map_err(|_| Error::from(ErrorKind::InvalidFile))?;
    let modify = serde_json::from_str::<Modify>(&modify_str)
        .map_err(|_| Error::from(ErrorKind::ModifyDecodeError))?;
    Ok(modify)
}

/// Decode the buffer of ncm file.
/// Return a Result containing a Vec<u8>.
/// You can write it to a file.
///
/// # Example
///
/// ```
/// extern crate ncmdump;
///
/// use std::error::Error;
/// use std::fs::{read, write};
/// use std::path::Path;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let input_path = Path::new("tests/test.ncm");
///     let output_path = Path::new("tests/test.flac");
///     let buffer = read(&input_path)?;
///     let data = ncmdump::decode(&buffer)?;
///     write(&output_path, data)?;
///     Ok(())
/// }
/// ```
pub fn decode(file_buffer: &[u8]) -> Result<Vec<u8>, Error> {
    let blocks = get_blocks(file_buffer)?;
    let key = get_key(&blocks.key)?;
    let data = get_data(&key, &blocks.data);
    Ok(data)
}

/// Get modify information from a buffer of ncm file.
/// Return a Result containing a Modify struct.
///
/// # Example
///
/// ```
/// extern crate ncmdump;
///
/// use std::error::Error;
/// use std::fs::read;
/// use std::path::Path;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let input_path = Path::new("tests/test.ncm");
///     let buffer = read(&input_path)?;
///     let modify = ncmdump::get_info(&buffer)?;
///     println!("{:?}", modify);
///     Ok(())
/// }
/// ```
pub fn get_info(file_buffer: &[u8]) -> Result<Modify, Error> {
    let blocks = get_blocks(file_buffer)?;
    let modify = get_modify(&blocks.modify)?;
    Ok(modify)
}
