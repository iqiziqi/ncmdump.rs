use anyhow::Result;
use std::io::{Read, Write};

const BUFFER_SIZE: usize = 8192;
const KEY: [u8; 256] = [
    0x77, 0x48, 0x32, 0x73, 0xDE, 0xF2, 0xC0, 0xC8, 0x95, 0xEC, 0x30, 0xB2, 0x51, 0xC3, 0xE1, 0xA0,
    0x9E, 0xE6, 0x9D, 0xCF, 0xFA, 0x7F, 0x14, 0xD1, 0xCE, 0xB8, 0xDC, 0xC3, 0x4A, 0x67, 0x93, 0xD6,
    0x28, 0xC2, 0x91, 0x70, 0xCA, 0x8D, 0xA2, 0xA4, 0xF0, 0x08, 0x61, 0x90, 0x7E, 0x6F, 0xA2, 0xE0,
    0xEB, 0xAE, 0x3E, 0xB6, 0x67, 0xC7, 0x92, 0xF4, 0x91, 0xB5, 0xF6, 0x6C, 0x5E, 0x84, 0x40, 0xF7,
    0xF3, 0x1B, 0x02, 0x7F, 0xD5, 0xAB, 0x41, 0x89, 0x28, 0xF4, 0x25, 0xCC, 0x52, 0x11, 0xAD, 0x43,
    0x68, 0xA6, 0x41, 0x8B, 0x84, 0xB5, 0xFF, 0x2C, 0x92, 0x4A, 0x26, 0xD8, 0x47, 0x6A, 0x7C, 0x95,
    0x61, 0xCC, 0xE6, 0xCB, 0xBB, 0x3F, 0x47, 0x58, 0x89, 0x75, 0xC3, 0x75, 0xA1, 0xD9, 0xAF, 0xCC,
    0x08, 0x73, 0x17, 0xDC, 0xAA, 0x9A, 0xA2, 0x16, 0x41, 0xD8, 0xA2, 0x06, 0xC6, 0x8B, 0xFC, 0x66,
    0x34, 0x9F, 0xCF, 0x18, 0x23, 0xA0, 0x0A, 0x74, 0xE7, 0x2B, 0x27, 0x70, 0x92, 0xE9, 0xAF, 0x37,
    0xE6, 0x8C, 0xA7, 0xBC, 0x62, 0x65, 0x9C, 0xC2, 0x08, 0xC9, 0x88, 0xB3, 0xF3, 0x43, 0xAC, 0x74,
    0x2C, 0x0F, 0xD4, 0xAF, 0xA1, 0xC3, 0x01, 0x64, 0x95, 0x4E, 0x48, 0x9F, 0xF4, 0x35, 0x78, 0x95,
    0x7A, 0x39, 0xD6, 0x6A, 0xA0, 0x6D, 0x40, 0xE8, 0x4F, 0xA8, 0xEF, 0x11, 0x1D, 0xF3, 0x1B, 0x3F,
    0x3F, 0x07, 0xDD, 0x6F, 0x5B, 0x19, 0x30, 0x19, 0xFB, 0xEF, 0x0E, 0x37, 0xF0, 0x0E, 0xCD, 0x16,
    0x49, 0xFE, 0x53, 0x47, 0x13, 0x1A, 0xBD, 0xA4, 0xF1, 0x40, 0x19, 0x60, 0x0E, 0xED, 0x68, 0x09,
    0x06, 0x5F, 0x4D, 0xCF, 0x3D, 0x1A, 0xFE, 0x20, 0x77, 0xE4, 0xD9, 0xDA, 0xF9, 0xA4, 0x2B, 0x76,
    0x1C, 0x71, 0xDB, 0x00, 0xBC, 0xFD, 0x0C, 0x6C, 0xA5, 0x47, 0xF7, 0xF6, 0x00, 0x79, 0x4A, 0x11,
];

pub struct QmcDump<S>(S)
where
    S: Read;

impl<S> QmcDump<S>
where
    S: Read,
{
    fn map_l(value: usize) -> Result<u8> {
        let v = if value > 0x7FFF {
            value % 0x7FFF
        } else {
            value
        };
        let index = (v * v + 80923) % 256;
        Ok(KEY[index])
    }

    fn encrypt(offset: usize, buffer: &mut [u8]) -> Result<()> {
        for index in 0..buffer.len() {
            buffer[index] ^= Self::map_l(offset + index)?;
        }
        Ok(())
    }

    /// Create QmcDump from reader.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::fs::File;
    /// #
    /// # use ncmdump::QmcDump;
    /// #
    /// let file = File::open("tests/test.qmcflac").expect("Can't open file");
    /// let ncm = QmcDump::from_reader(file).unwrap();
    /// ```
    pub fn from_reader(reader: S) -> Result<Self> {
        Ok(Self(reader))
    }

    /// Get the music data from qmcdump.
    ///
    /// # Example:
    ///
    /// ```rust
    /// use std::fs::File;
    /// use std::io::Write;
    /// use std::path::Path;
    ///
    /// use anyhow::Result;
    /// use ncmdump::QmcDump;
    ///
    /// fn main() -> Result<()> {
    ///     let file = File::open("tests/test.qmcflac")?;
    ///     let mut qmc = QmcDump::from_reader(file)?;
    ///     let music = qmc.get_data()?;
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
        let mut buffer = [0; BUFFER_SIZE];
        let mut offset = 0;
        let mut output = Vec::new();
        loop {
            let size = self.0.read(&mut buffer)?;
            Self::encrypt(offset, &mut buffer)?;
            output.write_all(&buffer)?;
            offset += size;
            if size == 0 {
                break;
            }
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::fs::File;

    use super::*;

    #[test]
    fn test() -> Result<()> {
        let input = File::open("tests/test.qmcflac")?;
        let mut output = File::options()
            .create(true)
            .write(true)
            .open("tests/test.flac")?;
        let mut qmc = QmcDump::from_reader(input)?;
        let data = qmc.get_data()?;
        output.write_all(&data)?;
        Ok(())
    }
}