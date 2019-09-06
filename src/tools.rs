extern crate aes_soft;
extern crate base64;
extern crate block_modes;
extern crate serde;

use std::convert::TryInto;
use std::slice::Iter;

use aes_soft::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Ecb};
use serde::Deserialize;

use crate::error::{Error, ErrorKind};

type Aes128Ecb = Ecb<Aes128, Pkcs7>;

static HEADER_KEY: [u8; 16] = [
    0x68, 0x7A, 0x48, 0x52, 0x41, 0x6D, 0x73, 0x6F,
    0x35, 0x6B, 0x49, 0x6E, 0x62, 0x61, 0x78, 0x57,
];

static MODIFY_KEY: [u8; 16] = [
    0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21,
    0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C, 0x27, 0x28,
];

#[derive(Debug, Deserialize)]
pub struct Modify {
    #[serde(rename = "musicName")]
    pub name: String,
    #[serde(rename = "musicId")]
    pub id: u64,
    pub album: String,
    pub artist: Vec<(String, u64)>,
    pub bitrate: u64,
    pub duration: u64,
    pub format: String,

    #[serde(rename = "mvId")]
    pub mv_id: Option<u64>,
    pub alias: Option<Vec<String>>,
}

pub struct BlockInfo {
    pub key: Vec<u8>,
    pub modify: Vec<u8>,
    pub crc: Vec<u8>,
    pub image: Vec<u8>,
    pub data: Vec<u8>,
}

fn get_n_element(iter: &mut Iter<u8>, n: usize) -> Option<Vec<u8>> {
    let mut result: Vec<u8> = Vec::new();
    for _ in 0..n {
        if let Some(i) = iter.next() {
            result.push(*i);
        }
    }
    if result.is_empty() { return None; }
    Some(result)
}

fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    let cipher = Aes128Ecb::new_var(&key, &[]).unwrap();
    let result = cipher.decrypt_vec(data).unwrap();
    Ok(result)
}

fn check_format(buffer: &[u8]) -> Result<(), Error> {
    let (buf, _) = buffer.split_at(std::mem::size_of::<u64>());
    let temp = u64::from_ne_bytes(buf.try_into().unwrap());
    if temp != 0x4d41_4446_4e45_5443 {
        return Err(Error::from(ErrorKind::InvalidFile));
    }
    Ok(())
}

fn get_length(buffer: &[u8]) -> Result<u32, Error> {
    let bytes = buffer.try_into().unwrap();
    let length = u32::from_ne_bytes(bytes);
    Ok(length)
}

fn build_key_box(key: &[u8]) -> Vec<usize> {
    let mut last_byte = 0;
    let mut key_box = (0..256).collect::<Vec<usize>>();
    let mut offsets = (0..key.len()).cycle();
    for i in 0..256 {
        let offset = offsets.next().unwrap();
        let c = (key_box[i] + last_byte + key[offset] as usize) & 0xff;
        key_box.swap(i, c);
        last_byte = c;
    }
    key_box
}

pub fn get_blocks(file_buffer: &[u8]) -> Result<BlockInfo, Error> {

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

pub fn get_data(key: &[u8], data: &[u8]) -> Vec<u8> {
    let key_box = build_key_box(&key);
    data.chunks(0x8000)
        .map(|i| i.iter().enumerate().map(|(index, item)| {
            let j = (index + 1) & 0xff;
            item ^ key_box[(key_box[j] + key_box[(key_box[j] + j) & 0xff]) & 0xff] as u8
        }))
        .flatten()
        .collect::<Vec<u8>>()
}

pub fn get_key(buffer: &[u8]) -> Result<Vec<u8>, Error> {
    let key_buffer = buffer.iter()
        .map(|byte| byte ^ 0x64)
        .collect::<Vec<u8>>();
    let decrypt_buffer = decrypt(&key_buffer, &HEADER_KEY)?;
    Ok(decrypt_buffer[17..].to_vec())
}

pub fn get_modify(buffer: &[u8]) -> Result<Modify, Error> {
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
