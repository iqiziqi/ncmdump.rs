extern crate base64;
extern crate serde;

use std::convert::TryInto;
use std::slice::Iter;

use crate::error::{Error, ErrorKind};

pub fn get_n_element(iter: &mut Iter<u8>, n: usize) -> Option<Vec<u8>> {
    let mut result: Vec<u8> = Vec::new();
    for _ in 0..n {
        if let Some(i) = iter.next() {
            result.push(*i);
        }
    }
    if result.is_empty() {
        return None;
    }
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
