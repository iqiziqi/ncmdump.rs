use std::io::{Read, Seek, SeekFrom};

use anyhow::Result;

use crate::decrypt::{build_key_box, decrypt, HEADER_KEY, MODIFY_KEY};
use crate::error::Errors;
use crate::Modify;

pub struct Ncmdump<S>
where
    S: Read + Seek,
{
    reader: S,
    key: (u64, u64),
    modify: (u64, u64),
    image: (u64, u64),
}

impl<S> Ncmdump<S>
where
    S: Read + Seek,
{
    /// Check the file format by header.
    fn check_format(buffer: &[u8]) -> Result<bool> {
        let (buf, _) = buffer.split_at(std::mem::size_of::<u64>());
        let temp = u64::from_ne_bytes(buf.try_into()?);
        if temp != 0x4d41_4446_4e45_5443 {
            return Ok(false);
        }
        Ok(true)
    }

    /// Get length by byte buffer.
    fn get_length(buffer: &[u8]) -> Result<u64> {
        let bytes = buffer.try_into()?;
        let length = u32::from_ne_bytes(bytes) as u64;
        Ok(length)
    }

    /// Create a Ncmdump from a seekable reader.
    /// Usually, the reader is a `File` or `Cursor`.
    ///
    /// # Example
    ///
    /// From a file.
    ///
    /// ```rust
    /// # use std::fs::File;
    /// #
    /// # use ncmdump::Ncmdump;
    /// #
    /// let file = File::open("tests/test.ncm").expect("Can't open file");
    /// let ncm = Ncmdump::from_reader(file).unwrap();
    /// ```
    /// Or from a Cursor.
    /// ```rust
    /// # use std::fs::File;
    /// # use std::io::{Cursor, Read};
    /// #
    /// # use ncmdump::Ncmdump;
    /// #
    /// # let mut file = File::open("tests/test.ncm").expect("Can't open file.");
    /// # let mut data = Vec::new();
    /// # file.read_to_end(&mut data).expect("Can't read file");
    /// let cursor = Cursor::new(data);
    /// let _ = Ncmdump::from_reader(cursor).unwrap();
    /// ```
    pub fn from_reader(mut reader: S) -> Result<Self> {
        // check format
        let mut format = [0; 10];
        let size = reader.read(&mut format)?;
        if size != 10 || !Self::check_format(&format)? {
            return Err(Errors::InvalidFileType.into());
        }

        let mut key_length_buffer = [0; 4];
        let read_size = reader.read(&mut key_length_buffer)? as u64;
        if read_size != 4 {
            return Err(Errors::InvalidKeyLength.into());
        }
        let key_start = reader.stream_position()?;
        let key_length = Self::get_length(&key_length_buffer)?;

        reader.seek(SeekFrom::Current(key_length as i64))?;
        let mut modify_length_buffer = [0; 4];
        let read_size = reader.read(&mut modify_length_buffer)? as u64;
        if read_size != 4 {
            return Err(Errors::InvalidModifyLength.into());
        }
        let modify_start = reader.stream_position()?;
        let modify_length = Self::get_length(&modify_length_buffer)?;

        reader.seek(SeekFrom::Current(modify_length as i64))?;
        reader.seek(SeekFrom::Current(9))?;
        let mut image_length_buffer = [0; 4];
        let read_size = reader.read(&mut image_length_buffer)?;
        if read_size != 4 {
            return Err(Errors::InvalidImageLength.into());
        }
        let image_start = reader.stream_position()?;
        let image_length = Self::get_length(&image_length_buffer)?;

        Ok(Self {
            reader,
            key: (key_start, key_length),
            modify: (modify_start, modify_length),
            image: (image_start, image_length),
        })
    }

    /// Utils for get bytes.
    fn get_bytes(&mut self, start: u64, length: u64) -> Result<Vec<u8>> {
        let reader = self.reader.by_ref();
        let mut key = Vec::new();
        reader.seek(SeekFrom::Start(start))?;
        reader.take(length).read_to_end(&mut key)?;
        Ok(key)
    }

    /// Utils for get key.
    pub fn get_key(&mut self) -> Result<Vec<u8>> {
        let (start, length) = self.key;
        let key = self.get_bytes(start, length)?;
        let key_buffer = key.iter().map(|byte| byte ^ 0x64).collect::<Vec<u8>>();
        let decrypt_buffer = decrypt(&key_buffer, &HEADER_KEY)?;
        Ok(decrypt_buffer[17..].to_vec())
    }

    /// Decode the modify buffer and just return the file modify.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::fs::File;
    /// use std::path::Path;
    ///
    /// use anyhow::Result;
    /// use ncmdump::Ncmdump;
    ///
    /// fn main() -> Result<()> {
    ///     let file = File::open("tests/test.ncm")?;
    ///     let mut ncm = Ncmdump::from_reader(file)?;
    ///     let modify = ncm.get_info();
    ///     println!("{:?}", modify);
    ///     Ok(())
    /// }
    /// ```
    pub fn get_info(&mut self) -> Result<Modify> {
        let (start, length) = self.modify;
        let modify = self.get_bytes(start, length)?;
        let modify_tmp = modify.iter().map(|item| item ^ 0x63).collect::<Vec<u8>>();
        let modify_key = base64::decode(&modify_tmp[22..]).map_err(|_| Errors::InvalidFileType)?;
        let modify_data = decrypt(&modify_key, &MODIFY_KEY)?;
        let modify_str =
            String::from_utf8(modify_data[6..].to_vec()).map_err(|_| Errors::ModifyDecodeError)?;
        let modify =
            serde_json::from_str::<Modify>(&modify_str).map_err(|_| Errors::ModifyDecodeError)?;
        Ok(modify)
    }

    /// Get the image bytes from ncmdump, if it's exists.
    ///
    /// # Example:
    ///
    /// ```rust
    /// use std::fs::File;
    /// use std::path::Path;
    ///
    /// use anyhow::Result;
    /// use ncmdump::Ncmdump;
    ///
    /// fn main() -> Result<()> {
    ///     use std::io::Write;
    /// let file = File::open("tests/test.ncm")?;
    ///     let mut ncm = Ncmdump::from_reader(file)?;
    ///     let image = ncm.get_image()?;
    ///
    ///     let mut target = File::options()
    ///         .create(true)
    ///         .write(true)
    ///         .open("tests/test.jpeg")?;
    ///     target.write_all(&image)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn get_image(&mut self) -> Result<Vec<u8>> {
        let (start, end) = self.image;
        let image = self.get_bytes(start, end)?;
        Ok(image)
    }

    /// Get the music data from ncmdump.
    ///
    /// # Example:
    ///
    /// ```rust
    /// use std::fs::File;
    /// use std::path::Path;
    ///
    /// use anyhow::Result;
    /// use ncmdump::Ncmdump;
    ///
    /// fn main() -> Result<()> {
    ///     use std::io::Write;
    /// let file = File::open("tests/test.ncm")?;
    ///     let mut ncm = Ncmdump::from_reader(file)?;
    ///     let music = ncm.get_data()?;
    ///
    ///     let mut target = File::options()
    ///         .create(true)
    ///         .write(true)
    ///         .open("tests/test.flac")?;
    ///     target.write_all(&music)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn get_data(&mut self) -> Result<Vec<u8>> {
        let start = self.image.0 + self.image.1;
        let mut data = Vec::new();
        self.reader.seek(SeekFrom::Start(start))?;
        self.reader.read_to_end(&mut data)?;
        let key = self.get_key()?;
        let key_box = build_key_box(&key);
        let result = data
            .chunks(0x8000)
            .flat_map(|i| {
                i.iter().enumerate().map(|(index, item)| {
                    let j = (index + 1) & 0xff;
                    item ^ key_box[(key_box[j] + key_box[(key_box[j] + j) & 0xff]) & 0xff] as u8
                })
            })
            .collect::<Vec<u8>>();
        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use std::fs::File;

    use anyhow::Result;

    use super::*;
    use crate::{get_blocks, get_key, get_modify};

    #[test]
    fn test_create_dump_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let _ = Ncmdump::from_reader(reader)?;
        Ok(())
    }

    #[test]
    fn test_get_key_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let key_new = ncm.get_key()?;

        let mut reader = File::open("./tests/test.ncm")?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        let blocks = get_blocks(&data)?;
        let key_old = get_key(blocks.key)?;

        assert_eq!(key_new, key_old);
        Ok(())
    }

    #[test]
    fn test_get_modify_ok() -> Result<()> {
        let mut reader = File::open("./tests/test.ncm")?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        let blocks = get_blocks(&data)?;
        let modify_old = get_modify(blocks.modify)?;

        let reader = File::open("./tests/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let modify_new = ncm.get_info()?;

        assert_eq!(modify_new, modify_old);
        Ok(())
    }

    #[test]
    fn test_get_image_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let image_new = ncm.get_image()?;

        let mut reader = File::open("./tests/test.ncm")?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        let blocks = get_blocks(&data)?;
        let image_old = blocks.image.to_vec();

        assert_eq!(image_new, image_old);
        Ok(())
    }
}
