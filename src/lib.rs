extern crate aes_soft;
extern crate base64;
extern crate block_modes;
extern crate serde;

mod error;
mod tools;

use crate::tools::{
    get_blocks,
    get_modify,
    get_key,
    get_data,
};

pub use tools::{Modify};
pub use error::{Error, ErrorKind};

pub fn process(file_buffer: &[u8]) -> Result<Vec<u8>, Error> {
    let blocks = get_blocks(file_buffer)?;
    let key = get_key(&blocks.key)?;
    let data = get_data(&key, &blocks.data);
    Ok(data)
}

pub fn get_info(file_buffer: &[u8]) -> Result<Modify, Error> {
    let blocks = get_blocks(file_buffer)?;
    let modify = get_modify(&blocks.modify)?;
    Ok(modify)
}
