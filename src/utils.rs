extern crate base64;
extern crate serde;

use std::convert::TryInto;
use std::slice::Iter;
use serde::Deserialize;

use crate::error::{Error, ErrorKind};

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


pub fn check_format(buffer: &[u8]) -> Result<(), Error> {
    let (buf, _) = buffer.split_at(std::mem::size_of::<u64>());
    let temp = u64::from_ne_bytes(buf.try_into().unwrap());
    if temp != 0x4d41_4446_4e45_5443 {
        return Err(Error::from(ErrorKind::InvalidFile));
    }
    Ok(())
}

pub fn get_length(buffer: &[u8]) -> Result<u32, Error> {
    let bytes = buffer.try_into().unwrap();
    let length = u32::from_ne_bytes(bytes);
    Ok(length)
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
