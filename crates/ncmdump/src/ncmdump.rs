use std::io::{Read, Seek, SeekFrom, Write};

use aes::Aes128;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use cipher::{BlockDecryptMut, KeyInit};
use cipher::block_padding::Pkcs7;
use serde::{Deserialize, Serialize};

use crate::error::{Errors, Result};

const HEADER_KEY: [u8; 16] = [
    0x68, 0x7A, 0x48, 0x52, 0x41, 0x6D, 0x73, 0x6F, 0x35, 0x6B, 0x49, 0x6E, 0x62, 0x61, 0x78, 0x57,
];

const INFO_KEY: [u8; 16] = [
    0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21, 0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C, 0x27, 0x28,
];

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum NcmId {
    String(String),
    Integer(u64),
}

/// The ncm file information.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RawNcmInfo {
    /// The name of music
    #[serde(rename = "musicName")]
    pub name: String,
    /// The id of music
    #[serde(rename = "musicId")]
    pub id: NcmId,
    /// The album of music, it's an url
    pub album: String,
    /// The artist of music, first item is name, second item is id
    pub artist: Vec<(String, NcmId)>,
    // The bit rate of music
    pub bitrate: NcmId,
    /// The duration of music
    pub duration: NcmId,
    /// The format of music, is maybe 'mp3' or 'flac'
    pub format: String,
    /// The id of MV
    #[serde(rename = "mvId")]
    pub mv_id: Option<NcmId>,
    /// The alias of music
    pub alias: Option<Vec<String>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct NcmInfo {
    pub name: String,
    /// The id of music
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
    pub mv_id: Option<u64>,
    /// The alias of music
    pub alias: Option<Vec<String>>,
}

/// The ncm file dump wrapper.
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

impl From<RawNcmInfo> for NcmInfo {
    fn from(raw_info: RawNcmInfo) -> Self {
        Self {
            name: raw_info.name,
            id: raw_info.id.get_id().unwrap_or(0),
            album: raw_info.album,
            artist: raw_info
                .artist
                .into_iter()
                .map(|(name, id)| (name, id.get_id().unwrap_or(0)))
                .collect::<Vec<(String, u64)>>(),
            bitrate: raw_info.bitrate.get_id().unwrap_or(0),
            duration: raw_info.duration.get_id().unwrap_or(0),
            format: raw_info.format,
            mv_id: match raw_info.mv_id {
                Some(id) => match id.get_id() {
                    Ok(inner) => Some(inner),
                    Err(_) => None,
                },
                None => None,
            },
            alias: raw_info.alias,
        }
    }
}

impl NcmId {
    pub fn get_id(self) -> Result<u64> {
        match self {
            NcmId::String(s) => {
                if s.is_empty() {
                    return Err(Errors::InfoDecodeError);
                }
                s.parse().map_err(|_| Errors::InfoDecodeError)
            }
            NcmId::Integer(num) => Ok(num),
        }
    }
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
        let decrypt_buffer = Self::decrypt(&key_buffer, &HEADER_KEY)?;
        Ok(decrypt_buffer[17..].to_vec())
    }

    fn encrypt(&mut self, offset: u64, buffer: &mut [u8]) {
        for (index, byte) in buffer.iter_mut().enumerate() {
            let j = ((offset + index as u64 + 1) & 0xff) as usize;
            let key_index = (self.key_box[j] + self.key_box[(self.key_box[j] + j) & 0xff]) & 0xff;
            *byte ^= self.key_box[key_index] as u8;
        }
    }

    fn decrypt(data: &[u8], key: &[u8; 16]) -> Result<Vec<u8>> {
        let result = Aes128::new(key.into())
            .decrypt_padded_vec_mut::<Pkcs7>(data)
            .map_err(|_| Errors::DecryptError)?;
        Ok(result)
    }

    fn build_key_box(key: &[u8]) -> Vec<usize> {
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

    /// Check the file format by header.
    fn check_format(buffer: &[u8]) -> Result<bool> {
        let buf = buffer.split_at(std::mem::size_of::<u64>()).0;
        let bytes = buf.try_into().map_err(|_| Errors::Decode)?;
        let temp = u64::from_ne_bytes(bytes);
        if temp != 0x4d41_4446_4e45_5443 {
            return Ok(false);
        }
        Ok(true)
    }

    /// Get length by byte buffer.
    fn get_length(buffer: &[u8]) -> Result<u64> {
        let bytes = buffer.try_into().map_err(|_| Errors::Decode)?;
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
    /// let file = File::open("res/test.ncm").expect("Can't open file");
    /// let _ = Ncmdump::from_reader(file).unwrap();
    /// ```
    /// Or from a Cursor.
    /// ```rust
    /// # use std::fs::File;
    /// # use std::io::{Cursor, Read};
    /// #
    /// # use ncmdump::Ncmdump;
    /// #
    /// # let mut file = File::open("res/test.ncm").expect("Can't open file.");
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
            return Err(Errors::InvalidFileType);
        }

        let mut key_length_buffer = [0; 4];
        let read_size = reader.read(&mut key_length_buffer)? as u64;
        if read_size != 4 {
            return Err(Errors::InvalidKeyLength);
        }
        let key_start = reader.stream_position()?;
        let key_length = Self::get_length(&key_length_buffer)?;
        let mut key = Vec::new();
        let key_reader = reader.by_ref();
        key_reader.seek(SeekFrom::Start(key_start))?;
        let key_size = key_reader.take(key_length).read_to_end(&mut key)?;
        if key_length != key_size as u64 {
            return Err(Errors::InvalidKeyLength);
        }
        let key = Self::get_key(&key)?;
        let key_box = Self::build_key_box(&key);

        // reader.seek(SeekFrom::Current(key_length as i64))?;
        let mut info_length_buffer = [0; 4];
        let read_size = reader.read(&mut info_length_buffer)? as u64;
        if read_size != 4 {
            return Err(Errors::InvalidInfoLength);
        }
        let info_start = reader.stream_position()?;
        let info_length = Self::get_length(&info_length_buffer)?;

        reader.seek(SeekFrom::Current(info_length as i64))?;
        reader.seek(SeekFrom::Current(9))?;
        let mut image_length_buffer = [0; 4];
        let read_size = reader.read(&mut image_length_buffer)?;
        if read_size != 4 {
            return Err(Errors::InvalidImageLength);
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
    ///     let file = File::open("res/test.ncm")?;
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
        let info_data = Self::decrypt(&info_key, &INFO_KEY)?;
        let info_str =
            String::from_utf8(info_data[6..].to_vec()).map_err(|_| Errors::InfoDecodeError)?;
        let info =
            serde_json::from_str::<RawNcmInfo>(&info_str).map_err(|_| Errors::InfoDecodeError)?;
        Ok(NcmInfo::from(info))
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
    /// let file = File::open("res/test.ncm")?;
    ///     let mut ncm = Ncmdump::from_reader(file)?;
    ///     let image = ncm.get_image()?;
    ///
    ///     let mut target = File::options()
    ///         .create(true)
    ///         .write(true)
    ///         .open("res/test.jpeg")?;
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
    ///     let file = File::open("res/test.ncm")?;
    ///     let mut ncm = Ncmdump::from_reader(file)?;
    ///     let music = ncm.get_data()?;
    ///
    ///     let mut target = File::options()
    ///         .create(true)
    ///         .write(true)
    ///         .open("res/test.flac")?;
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
        let reader = File::open("res/test.ncm")?;
        let _ = Ncmdump::from_reader(reader)?;
        Ok(())
    }

    #[test]
    fn test_get_info_ok() -> Result<()> {
        let reader = File::open("res/test.ncm")?;
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
    fn test_ncm_info_convert_ok() {
        let info = NcmInfo::from(RawNcmInfo {
            name: "".to_string(),
            id: NcmId::String(String::from("")),
            album: "".to_string(),
            artist: vec![],
            bitrate: NcmId::String(String::from("")),
            duration: NcmId::String(String::from("")),
            format: "".to_string(),
            mv_id: None,
            alias: None,
        });
        assert_eq!(info.id, 0);
        assert_eq!(info.artist, Vec::new());
        assert_eq!(info.bitrate, 0);
        assert_eq!(info.duration, 0);
    }

    #[test]
    fn test_ncm_info_deserialize_golden_hour_ok() -> Result<()> {
        let raw = serde_json::from_slice::<RawNcmInfo>(
            br#"{
    "musicId": 1958557540,
    "musicName": "golden hour",
    "artist": [
        [
            "JVKE",
            32988392
        ]
    ],
    "albumId": 152231212,
    "album": "this is what ____ feels like (Vol. 1-4)",
    "albumPicDocId": "109951167909857256",
    "albumPic": "https://p3.music.126.net/xKRQRZxEClE6653o7NbHiw==/109951167909857256.jpg",
    "bitrate": 320000,
    "mp3DocId": "d919d45cba30d6d5e5daff43b71cca8a",
    "duration": 209259,
    "mvId": 14570752,
    "alias": [],
    "transNames": [],
    "format": "mp3",
    "flag": 260
}"#,
        )?;
        let info = NcmInfo::from(raw);
        assert_eq!(info.id, 1958557540);
        assert_eq!(info.artist, vec!((String::from("JVKE"), 32988392),));
        assert_eq!(info.bitrate, 320000);
        assert_eq!(info.duration, 209259);
        assert_eq!(info.mv_id, Some(14570752));
        Ok(())
    }

    #[test]
    fn test_ncm_info_deserialize_endless_summer_ok() -> Result<()> {
        let raw = serde_json::from_slice::<RawNcmInfo>(
            br#"{
    "musicId": "2062898402",
    "musicName": "Endless Summer",
    "artist": [
        [
            "Alan Walker",
            "1045123"
        ],
        [
            "Zak Abel",
            "1053190"
        ]
    ],
    "albumId": "178429151",
    "album": "Walkerworld",
    "albumPicDocId": "109951169035717968",
    "albumPic": "http://p3.music.126.net/V4y-2EL08KAaHWLa5wYJFw==/109951169035717968.jpg",
    "bitrate": 320000,
    "mp3DocId": "d0e253a78e474075cced9a9f7a35498c",
    "duration": 187040,
    "mvId": "14648342",
    "transNames": [],
    "format": "mp3",
    "fee": 8,
    "volumeDelta": -10.2386,
    "privilege": {
        "flag": 1277956
    }
}"#,
        )?;
        let info = NcmInfo::from(raw);
        assert_eq!(info.id, 2062898402);
        assert_eq!(
            info.artist,
            vec!(
                (String::from("Alan Walker"), 1045123),
                (String::from("Zak Abel"), 1053190)
            )
        );
        assert_eq!(info.bitrate, 320000);
        assert_eq!(info.duration, 187040);
        assert_eq!(info.mv_id, Some(14648342));
        Ok(())
    }

    #[test]
    fn test_ncm_info_deserialize_eternal_regret_ok() -> Result<()> {
        let raw = serde_json::from_slice::<RawNcmInfo>(
            br#"{
   "musicId": "2056228066",
   "musicName": "Eternal regret",
   "artist": [
      [
         "AnRain",
         "35516167"
      ]
   ],
   "albumId": "167614757",
   "album": "Eternal regret",
   "albumPicDocId": "109951168681303606",
   "albumPic": "http://p4.music.126.net/XfUBgbH9RDQqBcStBVa-Pw==/109951168681303606.jpg",
   "bitrate": 320000,
   "mp3DocId": "d27e546865e86910e1da2ccbefd00a15",
   "duration": 186741,
   "mvId": "",
   "alias": [],
   "transNames": [],
   "format": "mp3",
   "fee": 8,
   "volumeDelta": -8.0989,
   "privilege": {
      "flag": 1544198
   }
}"#,
        )?;
        let info = NcmInfo::from(raw);
        assert_eq!(info.id, 2056228066);
        assert_eq!(info.artist, vec!((String::from("AnRain"), 35516167)));
        assert_eq!(info.bitrate, 320000);
        assert_eq!(info.duration, 186741);
        assert_eq!(info.mv_id, None);
        Ok(())
    }

    #[test]
    fn test_get_image_ok() -> Result<()> {
        let reader = File::open("res/test.ncm")?;
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
        let reader = File::open("res/test.ncm")?;
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
        let reader = File::open("res/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let mut data = [63, 246, 41, 107];
        ncm.encrypt(0, &mut data);
        assert_eq!(data, [102, 76, 97, 67]);
        Ok(())
    }

    #[test]
    fn test_ncmdump_read_ok() -> Result<()> {
        let reader = File::open("res/test.ncm")?;
        let mut ncm = Ncmdump::from_reader(reader)?;
        let mut buf = [0; 4];

        let size = ncm.read(&mut buf)?;
        assert_eq!(size, 4);
        assert_eq!(buf, [0x66, 0x4c, 0x61, 0x43]);
        Ok(())
    }

    #[test]
    fn test_ncmdump_multi_read_ok() -> Result<()> {
        let reader = File::open("res/test.ncm")?;
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

    #[test]
    fn test_decrypt() {
        let key = [
            0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21, 0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C,
            0x27, 0x28,
        ];
        let source = [
            0x2F, 0xAE, 0x05, 0x53, 0x75, 0xB8, 0x63, 0x15, 0xF7, 0x64, 0x54, 0xE6, 0x6D, 0x9F,
            0xBC, 0xFD, 0xD4, 0xC3, 0xE5, 0x4A, 0x5D, 0x7D, 0x61, 0x47, 0x93, 0x82, 0x00, 0xAC,
            0x48, 0xE4, 0x2C, 0x96, 0x43, 0x0F, 0x53, 0x54, 0xBE, 0x36, 0xC8, 0x6D, 0xE5, 0x46,
            0x47, 0x7E, 0x87, 0x88, 0xBC, 0xF6, 0x0F, 0x97, 0x0F, 0xA9, 0x60, 0xDB, 0xF7, 0x4C,
            0xD1, 0xD0, 0xD4, 0x44, 0xEC, 0xF6, 0x89, 0x44, 0x6B, 0x88, 0xD3, 0x2A, 0x39, 0x25,
            0xB8, 0x09, 0x71, 0xA9, 0x3F, 0xE0, 0xC1, 0x07, 0x22, 0x3A, 0xB6, 0x0B, 0xF0, 0xA1,
            0xB3, 0x26, 0x96, 0x62, 0xBD, 0x30, 0x76, 0xB5, 0xDA, 0x03, 0x6B, 0xB0, 0x99, 0x43,
            0x2E, 0x7B, 0x8F, 0xD4, 0xD2, 0x9E, 0xFF, 0x37, 0x24, 0xB5, 0x25, 0x56, 0x6A, 0x0B,
            0x2B, 0x41, 0x19, 0x52, 0xF6, 0xC8, 0x6E, 0x56, 0xE8, 0xC4, 0xF0, 0xCA, 0xFC, 0x9F,
            0x09, 0xA4, 0xED, 0x51, 0x94, 0x7E, 0xDD, 0xE4, 0xF1, 0x3D, 0x63, 0xEB, 0x7A, 0xBB,
            0x8C, 0xE6, 0x8F, 0x42, 0xDA, 0xE1, 0x7F, 0x17, 0xB5, 0x06, 0xC8, 0x34, 0x1C, 0xD7,
            0x4C, 0x7F, 0xBE, 0x7B, 0xE8, 0x66, 0xB1, 0x0C, 0xB6, 0x57, 0x65, 0x78, 0xCF, 0xD0,
            0xBA, 0xCC, 0x78, 0x09, 0xC1, 0x0F, 0x50, 0xDE, 0x1A, 0x2A, 0x27, 0x5D, 0x83, 0x12,
            0xBB, 0x3A, 0xA5, 0x12, 0xFB, 0x54, 0xA2, 0xC9, 0x4D, 0x15, 0x46, 0x9F, 0x36, 0x8D,
            0xF6, 0x79, 0x98, 0xA7, 0x34, 0x7F, 0x84, 0x6D, 0xD9, 0xC7, 0x3C, 0x51, 0xA6, 0x8B,
            0x95, 0x42, 0x09, 0x13, 0xA9, 0xAE, 0x2C, 0xE4, 0x5F, 0x8A, 0x26, 0x67, 0x3B, 0xF5,
            0x01, 0x36, 0x0D, 0x20, 0xD0, 0x2C, 0xDE, 0xA1, 0xE2, 0x03, 0x62, 0xD3, 0xB8, 0x9F,
            0x65, 0xE8, 0xD7, 0xC1, 0x05, 0x88, 0x83, 0x68, 0x50, 0xDB, 0xC7, 0x0F, 0xE5, 0x79,
            0xF7, 0x3F, 0x37, 0x4A, 0xF4, 0xD9, 0x82, 0xB1, 0xB3, 0x04, 0x0B, 0xB0, 0xD8, 0x5C,
            0xFA, 0x03, 0x9C, 0x63, 0xFF, 0xCA, 0xE2, 0xE4, 0x57, 0x08, 0x2A, 0x05, 0x34, 0x16,
            0x03, 0xED, 0xC7, 0x85, 0xE0, 0x3C, 0x43, 0x53, 0x78, 0x8C, 0x88, 0x7F, 0x52, 0xE6,
            0x94, 0x5D, 0xC7, 0x9A, 0x21, 0xD0, 0xEA, 0x89, 0x7B, 0x09, 0xB0, 0xDA, 0xA2, 0x0B,
            0xEE, 0xC2, 0x66, 0x04, 0x2A, 0xA0, 0x9C, 0x28, 0xDC, 0xA9, 0xB4, 0x71, 0x90, 0xC8,
            0x9B, 0x00, 0x7B, 0xF5, 0x7A, 0x9C, 0xAF, 0x2B, 0x8A, 0x7B, 0x0F, 0x70, 0x7F, 0x44,
            0x01, 0x5A, 0xDB, 0x6D, 0x8E, 0x98, 0x3D, 0x4E, 0x14, 0x71, 0xC1, 0xB2, 0x0F, 0x66,
            0x8F, 0x14, 0x0A, 0x5C, 0x78, 0xE9, 0x16, 0xD2, 0x45, 0x4F, 0x1F, 0xE3, 0x3F, 0x0E,
            0xBD, 0x5D, 0x40, 0x75,
        ];
        let target = [
            0x6D, 0x75, 0x73, 0x69, 0x63, 0x3A, 0x7B, 0x22, 0x6D, 0x75, 0x73, 0x69, 0x63, 0x49,
            0x64, 0x22, 0x3A, 0x31, 0x33, 0x30, 0x35, 0x33, 0x36, 0x36, 0x35, 0x35, 0x36, 0x2C,
            0x22, 0x6D, 0x75, 0x73, 0x69, 0x63, 0x4E, 0x61, 0x6D, 0x65, 0x22, 0x3A, 0x22, 0xE5,
            0xAF, 0x92, 0xE9, 0xB8, 0xA6, 0xE5, 0xB0, 0x91, 0xE5, 0xB9, 0xB4, 0x22, 0x2C, 0x22,
            0x61, 0x6C, 0x69, 0x61, 0x73, 0x22, 0x3A, 0x5B, 0x22, 0xE7, 0x94, 0xB5, 0xE8, 0xA7,
            0x86, 0xE5, 0x89, 0xA7, 0xE3, 0x80, 0x8A, 0xE6, 0x96, 0x97, 0xE7, 0xA0, 0xB4, 0xE8,
            0x8B, 0x8D, 0xE7, 0xA9, 0xB9, 0xE3, 0x80, 0x8B, 0xE4, 0xB8, 0xBB, 0xE9, 0xA2, 0x98,
            0xE6, 0x9B, 0xB2, 0x22, 0x5D, 0x2C, 0x22, 0x61, 0x72, 0x74, 0x69, 0x73, 0x74, 0x22,
            0x3A, 0x5B, 0x5B, 0x22, 0xE5, 0x8D, 0x8E, 0xE6, 0x99, 0xA8, 0xE5, 0xAE, 0x87, 0x22,
            0x2C, 0x38, 0x36, 0x31, 0x37, 0x37, 0x37, 0x5D, 0x5D, 0x2C, 0x22, 0x61, 0x6C, 0x62,
            0x75, 0x6D, 0x49, 0x64, 0x22, 0x3A, 0x37, 0x32, 0x37, 0x30, 0x36, 0x37, 0x38, 0x38,
            0x2C, 0x22, 0x61, 0x6C, 0x62, 0x75, 0x6D, 0x22, 0x3A, 0x22, 0xE5, 0xAF, 0x92, 0xE9,
            0xB8, 0xA6, 0xE5, 0xB0, 0x91, 0xE5, 0xB9, 0xB4, 0x22, 0x2C, 0x22, 0x61, 0x6C, 0x62,
            0x75, 0x6D, 0x50, 0x69, 0x63, 0x44, 0x6F, 0x63, 0x49, 0x64, 0x22, 0x3A, 0x31, 0x30,
            0x39, 0x39, 0x35, 0x31, 0x31, 0x36, 0x33, 0x35, 0x32, 0x30, 0x36, 0x33, 0x38, 0x35,
            0x32, 0x32, 0x2C, 0x22, 0x61, 0x6C, 0x62, 0x75, 0x6D, 0x50, 0x69, 0x63, 0x22, 0x3A,
            0x22, 0x68, 0x74, 0x74, 0x70, 0x3A, 0x2F, 0x2F, 0x70, 0x33, 0x2E, 0x6D, 0x75, 0x73,
            0x69, 0x63, 0x2E, 0x31, 0x32, 0x36, 0x2E, 0x6E, 0x65, 0x74, 0x2F, 0x71, 0x52, 0x51,
            0x54, 0x53, 0x5F, 0x54, 0x72, 0x6F, 0x5A, 0x6F, 0x39, 0x53, 0x4C, 0x56, 0x35, 0x79,
            0x71, 0x70, 0x54, 0x35, 0x41, 0x3D, 0x3D, 0x2F, 0x31, 0x30, 0x39, 0x39, 0x35, 0x31,
            0x31, 0x36, 0x33, 0x35, 0x32, 0x30, 0x36, 0x33, 0x38, 0x35, 0x32, 0x32, 0x2E, 0x6A,
            0x70, 0x67, 0x22, 0x2C, 0x22, 0x6D, 0x76, 0x49, 0x64, 0x22, 0x3A, 0x30, 0x2C, 0x22,
            0x62, 0x69, 0x74, 0x72, 0x61, 0x74, 0x65, 0x22, 0x3A, 0x39, 0x32, 0x33, 0x33, 0x37,
            0x38, 0x2C, 0x22, 0x64, 0x75, 0x72, 0x61, 0x74, 0x69, 0x6F, 0x6E, 0x22, 0x3A, 0x33,
            0x31, 0x35, 0x31, 0x34, 0x36, 0x2C, 0x22, 0x66, 0x6F, 0x72, 0x6D, 0x61, 0x74, 0x22,
            0x3A, 0x22, 0x66, 0x6C, 0x61, 0x63, 0x22, 0x7D,
        ];
        let result = Ncmdump::<File>::decrypt(&source, &key).unwrap();
        assert_eq!(&result[..], &target);
    }

    #[test]
    fn test_build_key_box() {
        let key = [
            0x31, 0x31, 0x38, 0x31, 0x39, 0x38, 0x30, 0x33, 0x33, 0x32, 0x38, 0x35, 0x45, 0x37,
            0x66, 0x54, 0x34, 0x39, 0x78, 0x37, 0x64, 0x6F, 0x66, 0x39, 0x4F, 0x4B, 0x43, 0x67,
            0x67, 0x39, 0x63, 0x64, 0x76, 0x68, 0x45, 0x75, 0x65, 0x7A, 0x79, 0x33, 0x69, 0x5A,
            0x43, 0x4C, 0x31, 0x6E, 0x46, 0x76, 0x42, 0x46, 0x64, 0x31, 0x54, 0x34, 0x75, 0x53,
            0x6B, 0x74, 0x41, 0x4A, 0x4B, 0x6D, 0x77, 0x5A, 0x58, 0x73, 0x69, 0x6A, 0x50, 0x62,
            0x69, 0x6A, 0x6C, 0x69, 0x69, 0x6F, 0x6E, 0x56, 0x55, 0x58, 0x58, 0x67, 0x39, 0x70,
            0x6C, 0x54, 0x62, 0x58, 0x45, 0x63, 0x6C, 0x41, 0x45, 0x39, 0x4C, 0x62,
        ];
        let key_box = vec![
            0x43, 0x63, 0x9D, 0xE2, 0x5B, 0x4B, 0x55, 0xBB, 0x4C, 0xCF, 0x2A, 0x62, 0x0E, 0x48,
            0x8A, 0x15, 0x59, 0x52, 0xBA, 0x6C, 0xEF, 0x6D, 0x72, 0x39, 0xA0, 0x9A, 0xA9, 0x27,
            0x66, 0xBC, 0xF9, 0xC0, 0x47, 0xDF, 0x7D, 0xDE, 0x3B, 0x81, 0x04, 0xFF, 0x90, 0x77,
            0x80, 0x50, 0x54, 0xBD, 0x0D, 0x58, 0x34, 0x0A, 0x44, 0xA8, 0x5F, 0x99, 0xC6, 0xBE,
            0x4E, 0x4D, 0x13, 0x17, 0x83, 0x01, 0x35, 0x5C, 0xF4, 0x7B, 0x53, 0x31, 0x86, 0xD4,
            0xB8, 0xAB, 0xD1, 0xB5, 0x68, 0xDC, 0x96, 0xF1, 0x9C, 0xE8, 0x7A, 0x1B, 0xB0, 0x56,
            0x22, 0x1A, 0x51, 0x92, 0xBF, 0xFA, 0xB1, 0x19, 0x88, 0x26, 0x49, 0x08, 0xEB, 0xAC,
            0x14, 0x28, 0xAD, 0x3A, 0x8C, 0x85, 0x84, 0x2C, 0x82, 0xB3, 0xA6, 0xA2, 0xA3, 0x12,
            0x78, 0xA1, 0x57, 0xAE, 0x00, 0x2F, 0xB6, 0x61, 0xA5, 0x6F, 0x5A, 0x89, 0x29, 0x46,
            0x2E, 0x4F, 0x36, 0x40, 0x07, 0x87, 0xA7, 0x65, 0x73, 0xC4, 0x7C, 0x33, 0x1E, 0xE5,
            0x10, 0xB4, 0xFD, 0xC9, 0xE0, 0xB7, 0x97, 0x32, 0x5D, 0x64, 0x41, 0xF0, 0x20, 0xC3,
            0x95, 0xFE, 0xD2, 0x21, 0xFB, 0x75, 0x3D, 0x0B, 0x3E, 0xF2, 0xD5, 0xCB, 0xD6, 0xF7,
            0x1F, 0x24, 0x45, 0x69, 0xB9, 0xDA, 0x6A, 0x76, 0x03, 0xF8, 0x70, 0x8E, 0xC1, 0xC8,
            0xD7, 0x4A, 0xD0, 0x9E, 0xCD, 0xA4, 0xCE, 0xAA, 0x1D, 0xED, 0xF6, 0x02, 0x60, 0xE3,
            0xDB, 0x8D, 0x09, 0xF3, 0x37, 0xE1, 0xC5, 0xCA, 0x8F, 0x2D, 0x7F, 0x74, 0x42, 0x6E,
            0x8B, 0x3F, 0x23, 0xC2, 0xD3, 0xCC, 0xD9, 0xEE, 0x98, 0xE6, 0x11, 0x05, 0xEA, 0xD8,
            0xB2, 0xE4, 0xF5, 0xE7, 0x71, 0x2B, 0x93, 0x9B, 0x3C, 0x30, 0xE9, 0xC7, 0x38, 0xEC,
            0x18, 0x6B, 0x79, 0xFC, 0xAF, 0x5E, 0x9F, 0x7E, 0x91, 0xDD, 0x16, 0x94, 0x0F, 0x06,
            0x67, 0x25, 0x0C, 0x1C,
        ];
        assert_eq!(Ncmdump::<File>::build_key_box(&key), key_box);
    }
}
