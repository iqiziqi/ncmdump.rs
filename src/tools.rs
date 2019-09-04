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
    name: String,
    #[serde(rename = "musicId")]
    id: u64,
    album: String,
    alias: Vec<String>,
    artist: Vec<(String, u64)>,
    bitrate: u64,
    duration: u64,
    format: String,
}

pub fn get_n_element(iter: &mut Iter<u8>, n: usize) -> Option<Vec<u8>> {
    let mut result: Vec<u8> = Vec::new();
    for _ in 0..n {
        if let Some(i) = iter.next() {
            result.push(*i);
        }
    }
    if result.is_empty() { return None; }
    Some(result)
}

pub fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    let cipher = Aes128Ecb::new_var(&key, &[]).unwrap();
    let a = cipher.decrypt_vec(data).unwrap();
    Ok(a)
}

pub fn check_format(buffer: &[u8]) -> Result<(), Error> {
    let (buf, _) = buffer.split_at(std::mem::size_of::<u64>());
    if u64::from_ne_bytes(buf.try_into().unwrap()) != 0x4d41_4446_4e45_5443 {
        return Err(Error::from(ErrorKind::InvalidFile));
    }
    Ok(())
}

pub fn get_length(buffer: &[u8]) -> Result<u32, Error> {
    let bytes = buffer.try_into().unwrap();
    let length = u32::from_ne_bytes(bytes);
    Ok(length)
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
        .map_err(|_| Error::from(ErrorKind::InvalidFile))?;
    Ok(modify)
}

pub fn get_image(buffer: Vec<u8>) -> Result<Vec<u8>, Error> {
    Ok(buffer)
}

pub fn build_key_box(key: &[u8]) -> Vec<usize> {
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
