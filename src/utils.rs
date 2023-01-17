use std::convert::TryInto;

use anyhow::Result;

use crate::error::Errors;

pub fn check_format(buffer: &[u8]) -> Result<()> {
    let (buf, _) = buffer.split_at(std::mem::size_of::<u64>());
    let temp = u64::from_ne_bytes(buf.try_into()?);
    if temp != 0x4d41_4446_4e45_5443 {
        return Err(Errors::InvalidFileType.into());
    }
    Ok(())
}

pub fn get_length(buffer: &[u8]) -> Result<usize> {
    let bytes = buffer.try_into()?;
    let length = u32::from_ne_bytes(bytes) as usize;
    Ok(length)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_check_format() -> Result<()> {
        let buffer = [67, 84, 69, 78, 70, 68, 65, 77];
        check_format(&buffer)?;
        Ok(())
    }

    #[test]
    fn test_check_format_panic() -> Result<()> {
        let buffer = [67, 84, 69, 78, 70, 68, 65, 76];
        let result = check_format(&buffer);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_get_length() -> Result<()> {
        assert_eq!(get_length(&[0x00, 0x00, 0x00, 0x00])?, 0);
        assert_eq!(get_length(&[0x4d, 0x41, 0x44, 0x46])?, 1_178_878_285);
        assert_eq!(get_length(&[0x4e, 0x45, 0x54, 0x43])?, 1_129_596_238);
        Ok(())
    }
}
