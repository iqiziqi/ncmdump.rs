extern crate aes_soft;
extern crate base64;
extern crate block_modes;
extern crate serde;

mod error;
mod tools;

use crate::tools::Modify;
use crate::tools::{
    get_length,
    get_image,
    build_key_box,
    get_modify,
    get_key,
    check_format,
    get_n_element,
};

pub use error::{Error, ErrorKind};

pub fn process(file_buffer: &[u8]) -> Result<Vec<u8>, Error> {

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
        let buffer = get_n_element(&mut iter, length as usize).unwrap();
        get_key(&buffer)?
    };

    // modify area
    {
        let buffer_length = get_n_element(&mut iter, 4).unwrap();
        let length = get_length(&buffer_length)?;
        get_n_element(&mut iter, length as usize).unwrap();
    };

    // crc area
    {
        get_n_element(&mut iter, 9).unwrap();
    };

    // image area
    {
        let buffer_length = get_n_element(&mut iter, 4).unwrap();
        let length = get_length(&buffer_length)?;
        let buffer = get_n_element(&mut iter, length as usize).unwrap();
        get_image(buffer)?;
    };

    // data area
    let data = {
        let key_box = build_key_box(&key);
        iter.as_slice().chunks(0x8000)
            .map(|i| i.iter().enumerate().map(|(index, item)| {
                let j = (index + 1) & 0xff;
                item ^ key_box[(key_box[j] + key_box[(key_box[j] + j) & 0xff]) & 0xff] as u8
            }))
            .flatten()
            .collect::<Vec<u8>>()
    };

    Ok(data)
}

pub fn get_info(file_buffer: &[u8]) -> Result<Modify, Error> {

    let mut iter = file_buffer.iter();

    {
        let buffer = get_n_element(&mut iter, 10).unwrap();
        check_format(&buffer)?;
    };

    let key = {
        let buffer_length = get_n_element(&mut iter, 4).unwrap();
        let length = get_length(&buffer_length)?;
        let buffer = get_n_element(&mut iter, length as usize).unwrap();
        get_key(&buffer)?
    };

    let modify = {
        let buffer_length = get_n_element(&mut iter, 4).unwrap();
        let length = get_length(&buffer_length)?;
        let buffer = get_n_element(&mut iter, length as usize).unwrap();
        get_modify(&buffer)?
    };

    Ok(modify)
}
