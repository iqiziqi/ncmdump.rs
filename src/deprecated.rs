use anyhow::Result;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::decrypt::{build_key_box, decrypt, HEADER_KEY, INFO_KEY};
use crate::error::Errors;
use crate::utils::{check_format, get_length};

/// The music modify information
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[deprecated(since = "0.3.0", note = "Use NcmInfo")]
#[allow(deprecated)]
pub struct Modify {
    /// The name of music
    #[serde(rename = "musicName")]
    pub name: String,
    /// The id of music
    #[serde(rename = "musicId")]
    pub id: u64,
    /// The album of music, it's an url
    pub album: String,
    /// The artist of music, first item is name, second item is id
    pub artist: Vec<(String, u64)>,
    // The bit rate of music
    pub bitrate: u64,
    /// The duration of music
    pub duration: u64,
    /// The format of music, is maybe 'mp3' or 'flac'
    pub format: String,
    /// The id of MV
    #[serde(rename = "mvId")]
    pub mv_id: Option<u64>,
    /// The alias of music
    pub alias: Option<Vec<String>>,
}

/// The block information.
#[derive(Debug)]
#[deprecated(since = "0.3.0", note = "Use Ncmdump")]
pub struct BlockInfo<'a> {
    /// The key block
    pub key: &'a [u8],
    /// The music info block
    pub modify: &'a [u8],
    /// The image block
    pub image: &'a [u8],
    /// The data block
    pub data: &'a [u8],
}

/// Get blocks by buffer
///
/// # Example:
///
/// ```rust
/// use std::fs::{read, write};
/// use std::path::Path;
///
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let file_path = Path::new("tests/test.ncm");
///     let buffer = read(&file_path)?;
///     let blocks = ncmdump::get_blocks(&buffer)?;
///     println!("{:?}", blocks);
///     Ok(())
/// }
/// ```
#[deprecated(since = "0.3.0", note = "Use Ncmdump::from_reader")]
#[allow(deprecated)]
pub fn get_blocks(file_buffer: &[u8]) -> Result<BlockInfo> {
    let (format_buffer, buffer) = file_buffer.split_at(10);
    check_format(format_buffer)?;

    // key area
    let (length_buffer, buffer) = buffer.split_at(4);
    let length = get_length(length_buffer)?;
    let (key, buffer) = buffer.split_at(length);

    // modify area
    let (length_buffer, buffer) = buffer.split_at(4);
    let length = get_length(length_buffer)?;
    let (modify, buffer) = buffer.split_at(length);

    // blank area
    let (_, buffer) = buffer.split_at(9);

    // image area
    let (buffer_length, buffer) = buffer.split_at(4);
    let length = get_length(buffer_length)?;
    let (image, data) = buffer.split_at(length);

    Ok(BlockInfo {
        key,
        modify,
        image,
        data,
    })
}

/// Get the music data.
///
/// # Example
///
/// ```rust
/// use std::fs::read;
/// use std::path::Path;
///
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let file_path = Path::new("tests/test.ncm");
///     let buffer = read(&file_path)?;
///     let blocks = ncmdump::get_blocks(&buffer)?;
///     let key = ncmdump::get_key(blocks.key)?;
///     let result = ncmdump::get_data(blocks.data, &key);
///     println!("{:?}", result);
///     Ok(())
/// }
/// ```
#[deprecated(since = "0.3.0", note = "Use Ncmdump::get_data")]
#[allow(deprecated)]
pub fn get_data(key: &[u8], data: &[u8]) -> Vec<u8> {
    let key_box = build_key_box(key);
    data.chunks(0x8000)
        .flat_map(|i| {
            i.iter().enumerate().map(|(index, item)| {
                let j = (index + 1) & 0xff;
                item ^ key_box[(key_box[j] + key_box[(key_box[j] + j) & 0xff]) & 0xff] as u8
            })
        })
        .collect::<Vec<u8>>()
}

/// Get music data key.
pub fn get_key(buffer: &[u8]) -> Result<Vec<u8>> {
    let key_buffer = buffer.iter().map(|byte| byte ^ 0x64).collect::<Vec<u8>>();
    let decrypt_buffer = decrypt(&key_buffer, &HEADER_KEY)?;
    Ok(decrypt_buffer[17..].to_vec())
}

/// Decode the music info buffer and just return the file modify.
///
/// # Example
///
/// ```rust
/// use std::fs::{read, write};
/// use std::path::Path;
///
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let file_path = Path::new("tests/test.ncm");
///     let buffer = read(&file_path)?;
///     let blocks = ncmdump::get_blocks(&buffer)?;
///     let modify = ncmdump::get_modify(blocks.modify)?;
///     println!("{:?}", modify);
///     Ok(())
/// }
/// ```
#[deprecated(since = "0.3.0", note = "Use Ncmdump::get_info")]
#[allow(deprecated)]
pub fn get_modify(buffer: &[u8]) -> Result<Modify> {
    let modify_tmp = buffer.iter().map(|item| item ^ 0x63).collect::<Vec<u8>>();
    let modify_key = STANDARD
        .decode(&modify_tmp[22..])
        .map_err(|_| Errors::InfoDecodeError)?;
    let modify_data = decrypt(&modify_key, &INFO_KEY)?;
    let modify_str =
        String::from_utf8(modify_data[6..].to_vec()).map_err(|_| Errors::InfoDecodeError)?;
    let modify =
        serde_json::from_str::<Modify>(&modify_str).map_err(|_| Errors::InfoDecodeError)?;
    Ok(modify)
}

/// The wrap of `get_data`. Decode the buffer of ncm file.
/// Return a Result containing a Vec<u8>.
/// You can write it to a file.
///
/// # Example
///
/// ```
/// use std::fs::{read, write};
/// use std::path::Path;
///
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let input_path = Path::new("tests/test.ncm");
///     let output_path = Path::new("tests/test.flac");
///     let buffer = read(&input_path)?;
///     let data = ncmdump::convert(&buffer)?;
///     write(&output_path, data)?;
///     Ok(())
/// }
/// ```
#[deprecated(since = "0.3.0", note = "Use Ncmdump::get_data")]
#[allow(deprecated)]
pub fn convert(file_buffer: &[u8]) -> Result<Vec<u8>> {
    let blocks = get_blocks(file_buffer)?;
    let key = get_key(blocks.key)?;
    let data = get_data(&key, blocks.data);
    Ok(data)
}

/// The wrap of `get_modify`. Get modify information from a buffer of ncm file.
/// Return a Result containing a Modify struct.
///
/// # Example
///
/// ```
/// use std::fs::read;
/// use std::path::Path;
///
/// use anyhow::Result;
///
/// fn main() -> Result<()> {
///     let input_path = Path::new("tests/test.ncm");
///     let buffer = read(&input_path)?;
///     let modify = ncmdump::get_info(&buffer)?;
///     println!("{:?}", modify);
///     Ok(())
/// }
/// ```
#[deprecated(since = "0.3.0", note = "Use Ncmdump::get_info")]
#[allow(deprecated)]
pub fn get_info(file_buffer: &[u8]) -> Result<Modify> {
    let blocks = get_blocks(file_buffer)?;
    let modify = get_modify(blocks.modify)?;
    Ok(modify)
}
