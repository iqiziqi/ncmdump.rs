extern crate aes_soft;
extern crate base64;
extern crate block_modes;
extern crate serde;

mod decrypt;
pub mod error;
pub mod utils;

use crate::decrypt::{HEADER_KEY, MODIFY_KEY, decrypt};
use crate::error::{Error, ErrorKind};
use crate::utils::{
    BlockInfo,
    Modify,
    build_key_box,
    check_format,
    get_n_element,
    get_length,
};

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

    Ok(BlockInfo {key, modify, crc, image, data})
}

fn get_data(key: &[u8], data: &[u8]) -> Vec<u8> {
    let key_box = build_key_box(&key);
    data.chunks(0x8000)
        .map(|i| i.iter().enumerate().map(|(index, item)| {
            let j = (index + 1) & 0xff;
            item ^ key_box[(key_box[j] + key_box[(key_box[j] + j) & 0xff]) & 0xff] as u8
        }))
        .flatten()
        .collect::<Vec<u8>>()
}

fn get_key(buffer: &[u8]) -> Result<Vec<u8>, Error> {
    let key_buffer = buffer.iter()
        .map(|byte| byte ^ 0x64)
        .collect::<Vec<u8>>();
    let decrypt_buffer = decrypt(&key_buffer, &HEADER_KEY)?;
    Ok(decrypt_buffer[17..].to_vec())
}

fn get_modify(buffer: &[u8]) -> Result<Modify, Error> {
    let modify_tmp = buffer.iter()
        .map(|item| item ^ 0x63)
        .collect::<Vec<u8>>();
    let modify_key = base64::decode(&modify_tmp[22..])
        .map_err(|_| Error::from(ErrorKind::InvalidFile))?;
    let modify_str = String::from_utf8(decrypt(&modify_key, &MODIFY_KEY)?[6..].to_vec())
        .map_err(|_| Error::from(ErrorKind::InvalidFile))?;
    let modify = serde_json::from_str::<Modify>(&modify_str)
        .map_err(|_| Error::from(ErrorKind::ModifyDecodeError))?;
    Ok(modify)
}

pub fn decode(file_buffer: &[u8]) -> Result<Vec<u8>, Error> {
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
