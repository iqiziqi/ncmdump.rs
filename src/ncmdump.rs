use std::io::{Read, Seek, SeekFrom, Write};

use anyhow::Result;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::decrypt::{build_key_box, decrypt, HEADER_KEY, INFO_KEY};
use crate::error::Errors;

/// The music information
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NcmInfo {
    /// The name of music
    #[serde(rename = "musicName")]
    pub name: String,
    /// The id of music
    #[serde(rename = "musicId")]
    pub id: u64,
    /// The album of music, it's an url
    pub album: String,
    /// The artist of music, first item is name, second item is id
    pub artist: Vec<(String, u64)>,
    // The bit rate of music
    pub bitrate: u64,
    /// The duration of music
    pub duration: u64,
    /// The format of music, is maybe 'mp3' or 'flac'
    pub format: String,
    /// The id of MV
    #[serde(rename = "mvId")]
    pub mv_id: Option<u64>,
    /// The alias of music
    pub alias: Option<Vec<String>>,
}

pub struct Ncmdump<S>
where
    S: Read,
{
    reader: S,
    cursor: u64,
    info: (u64, u64),
    image: (u64, u64),
    key_box: Vec<usize>,
}

impl<S> Ncmdump<S>
where
    S: Read,
{
    #[inline]
    fn base(&self) -> u64 {
        self.image.0 + self.image.1
    }

    fn get_key(key: &[u8]) -> Result<Vec<u8>> {
        let key_buffer = key.iter().map(|byte| byte ^ 0x64).collect::<Vec<u8>>();
        let decrypt_buffer = decrypt(&key_buffer, &HEADER_KEY)?;
        Ok(decrypt_buffer[17..].to_vec())
    }

    fn encrypt(&mut self, offset: u64, buffer: &mut [u8]) {
        for i in 0..buffer.len() {
            let j = ((offset + i as u64 + 1) & 0xff) as usize;
            let key_index = (self.key_box[j] + self.key_box[(self.key_box[j] + j) & 0xff]) & 0xff;
            buffer[i] ^= self.key_box[key_index] as u8;
        }
    }

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
}

impl<S> Ncmdump<S>
where
    S: Read + Seek,
{
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
        let mut key = Vec::new();
        let key_reader = reader.by_ref();
        key_reader.seek(SeekFrom::Start(key_start))?;
        let key_size = key_reader.take(key_length).read_to_end(&mut key)?;
        if key_length != key_size as u64 {
            return Err(Errors::InvalidKeyLength.into());
        }
        let key = Self::get_key(&key)?;
        let key_box = build_key_box(&key);

        // reader.seek(SeekFrom::Current(key_length as i64))?;
        let mut info_length_buffer = [0; 4];
        let read_size = reader.read(&mut info_length_buffer)? as u64;
        if read_size != 4 {
            return Err(Errors::InvalidInfoLength.into());
        }
        let info_start = reader.stream_position()?;
        let info_length = Self::get_length(&info_length_buffer)?;

        reader.seek(SeekFrom::Current(info_length as i64))?;
        reader.seek(SeekFrom::Current(9))?;
        let mut image_length_buffer = [0; 4];
        let read_size = reader.read(&mut image_length_buffer)?;
        if read_size != 4 {
            return Err(Errors::InvalidImageLength.into());
        }
        let image_start = reader.stream_position()?;
        let image_length = Self::get_length(&image_length_buffer)?;

        reader.seek(SeekFrom::Start(image_start + image_length))?;
        Ok(Self {
            reader,
            key_box,
            cursor: 0,
            info: (info_start, info_length),
            image: (image_start, image_length),
        })
    }

    /// Utils for get bytes.
    fn get_bytes(&mut self, start: u64, length: u64) -> Result<Vec<u8>> {
        let reader = self.reader.by_ref();
        let mut buf = Vec::new();
        reader.seek(SeekFrom::Start(start))?;
        reader.take(length).read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Decode the information buffer and just return the information.
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
    ///     let info = ncm.get_info();
    ///     println!("{:?}", info);
    ///     Ok(())
    /// }
    /// ```
    pub fn get_info(&mut self) -> Result<NcmInfo> {
        let (start, length) = self.info;
        let info_bytes = self.get_bytes(start, length)?;
        let info_tmp = info_bytes
            .iter()
            .map(|item| item ^ 0x63)
            .collect::<Vec<u8>>();
        let info_key = STANDARD
            .decode(&info_tmp[22..])
            .map_err(|_| Errors::InfoDecodeError)?;
        let info_data = decrypt(&info_key, &INFO_KEY)?;
        let info_str =
            String::from_utf8(info_data[6..].to_vec()).map_err(|_| Errors::InfoDecodeError)?;
        let info =
            serde_json::from_str::<NcmInfo>(&info_str).map_err(|_| Errors::InfoDecodeError)?;
        Ok(info)
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
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// use anyhow::Result;
    /// use ncmdump::Ncmdump;
    ///
    /// fn main() -> Result<()> {
    ///     let file = File::open("tests/test.ncm")?;
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
        let mut data = Vec::new();
        let mut buffer = [0; 0x8000];
        while let Ok(size) = self.read(&mut buffer) {
            if size == 0 {
                break;
            }
            data.write_all(&buffer[..size])?;
        }
        Ok(data)
    }
}

impl<R> Read for Ncmdump<R>
where
    R: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = self.reader.read(buf)?;
        self.encrypt(self.cursor, buf);
        self.cursor += size as u64;
        Ok(size)
    }
}

impl<R> Seek for Ncmdump<R>
where
    R: Read + Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let base = self.base();
        let pos = match pos {
            SeekFrom::Start(p) => SeekFrom::Start(p + base),
            _ => pos,
        };
        self.cursor = self.reader.seek(pos)? - base;
        Ok(self.cursor)
    }
}

#[cfg(test)]
pub mod tests {
    use std::fs::File;

    use anyhow::Result;

    use super::*;

    #[test]
    fn test_create_dump_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let _ = Ncmdump::from_reader(reader)?;
        Ok(())
    }

    #[test]
    fn test_get_info_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let info = ncm.get_info()?;

        assert_eq!(
            info,
            NcmInfo {
                name: "寒鸦少年".to_string(),
                id: 1305366556,
                album: "寒鸦少年".to_string(),
                artist: vec![("华晨宇".into(), 861777)],
                bitrate: 923378,
                duration: 315146,
                format: "flac".to_string(),
                mv_id: Some(0),
                alias: Some(vec!["电视剧《斗破苍穹》主题曲".into()]),
            },
        );
        Ok(())
    }

    #[test]
    fn test_get_image_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let image = ncm.get_image()?;
        let length = image.len();

        assert_eq!(length, 39009);
        assert_eq!(
            image[..16],
            [
                0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01,
                0x00, 0x48,
            ],
        );
        assert_eq!(
            image[38993..],
            [
                0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
                0xff, 0xd9,
            ],
        );

        Ok(())
    }

    #[test]
    fn test_get_data_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let data = ncm.get_data()?;
        let length = data.len();

        assert_eq!(length, 61440);
        assert_eq!(
            data[..16],
            [
                0x66, 0x4c, 0x61, 0x43, 0x00, 0x00, 0x00, 0x22, 0x12, 0x00, 0x12, 0x00, 0x00, 0x01,
                0x01, 0x00,
            ],
        );
        assert_eq!(
            data[61424..],
            [
                0x8b, 0x25, 0x88, 0x08, 0x4b, 0x49, 0x89, 0xc2, 0xba, 0xe3, 0xda, 0x88, 0x48, 0xc1,
                0x09, 0x7b,
            ],
        );
        Ok(())
    }

    #[test]
    fn test_encrypt_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let mut data = [63, 246, 41, 107];
        ncm.encrypt(0, &mut data);
        assert_eq!(data, [102, 76, 97, 67]);
        Ok(())
    }

    #[test]
    fn test_ncmdump_read_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let mut buf = [0; 4];

        let size = ncm.read(&mut buf)?;
        assert_eq!(size, 4);
        assert_eq!(buf, [0x66, 0x4c, 0x61, 0x43]);
        Ok(())
    }

    #[test]
    fn test_ncmdump_multi_read_ok() -> Result<()> {
        let reader = File::open("./tests/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let mut buf = [0; 4];

        let size = ncm.read(&mut buf)?;
        assert_eq!(size, 4);
        assert_eq!(buf, [0x66, 0x4c, 0x61, 0x43]);

        let size = ncm.read(&mut buf)?;
        assert_eq!(size, 4);
        assert_eq!(buf, [0x00, 0x00, 0x00, 0x22]);
        Ok(())
    }
}
