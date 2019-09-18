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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_n_element() {
        let collection = (0..99).collect::<Vec<u8>>();
        let mut iter: Iter<u8> = collection.iter();

        assert_eq!(get_n_element(&mut iter, 4).unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(get_n_element(&mut iter, 4).unwrap(), vec![4, 5, 6, 7]);
        assert_eq!(get_n_element(&mut iter, 2).unwrap(), vec![8, 9]);
    }

    #[test]
    fn test_check_format() {
        let buffer = [67, 84, 69, 78, 70, 68, 65, 77];
        check_format(&buffer).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_check_format_panic() {
        let buffer = [67, 84, 69, 78, 70, 68, 65, 76];
        check_format(&buffer).unwrap();
    }

    #[test]
    fn test_get_length() {
        assert_eq!(get_length(&[0x00, 0x00, 0x00, 0x00]).unwrap(), 0);
        assert_eq!(get_length(&[0x4d, 0x41, 0x44, 0x46]).unwrap(), 1_178_878_285);
        assert_eq!(get_length(&[0x4e, 0x45, 0x54, 0x43]).unwrap(), 1_129_596_238);
    }
}
