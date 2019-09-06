extern crate aes_soft;
extern crate block_modes;

use aes_soft::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Ecb};

use crate::error::Error;

type Aes128Ecb = Ecb<Aes128, Pkcs7>;

pub static HEADER_KEY: [u8; 16] = [
    0x68, 0x7A, 0x48, 0x52, 0x41, 0x6D, 0x73, 0x6F,
    0x35, 0x6B, 0x49, 0x6E, 0x62, 0x61, 0x78, 0x57,
];

pub static MODIFY_KEY: [u8; 16] = [
    0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21,
    0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C, 0x27, 0x28,
];

pub fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    let cipher = Aes128Ecb::new_var(&key, &[]).unwrap();
    let result = cipher.decrypt_vec(data).unwrap();
    Ok(result)
}
