use std::io::Read;

use crate::error::Result;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileType {
    /// The standard ncm file.
    #[cfg(feature = "ncmdump")]
    Ncm,
    /// The standard qmc file.
    #[cfg(feature = "qmcdump")]
    Qmc,
    /// The other file type.
    Other,
}

/// Return the file type of the reader.
pub fn get_file_type<R>(reader: &mut R) -> Result<FileType>
where
    R: Read,
{
    let mut head = [0; 8];
    if reader.read(&mut head)? != 8 {
        return Ok(FileType::Other);
    }

    let file_type = match head[..] {
        #[cfg(feature = "ncmdump")]
        [0x43, 0x54, 0x45, 0x4E, 0x46, 0x44, 0x41, 0x4D] => FileType::Ncm,
        #[cfg(feature = "qmcdump")]
        [0xA5, 0x06, 0xB7, 0x89, _, _, _, _] => FileType::Qmc,
        #[cfg(feature = "qmcdump")]
        [0x8A, 0x0E, 0xE5, _, _, _, _, _] => FileType::Qmc,
        _ => FileType::Other,
    };
    Ok(file_type)
}

/// Check if the reader is ncm format.
#[cfg(feature = "ncmdump")]
pub fn is_ncm_file<R>(reader: &mut R) -> Result<bool>
where
    R: Read,
{
    let file_type = get_file_type(reader)?;
    Ok(file_type == FileType::Ncm)
}

/// Check if the reader is qmc format.
#[cfg(feature = "qmcdump")]
pub fn is_qmc_file<R>(reader: &mut R) -> Result<bool>
where
    R: Read,
{
    let file_type = get_file_type(reader)?;
    Ok(file_type == FileType::Qmc)
}
