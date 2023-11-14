use std::fs::File;
use std::path::PathBuf;

use anyhow::Result;
use ncmdump::utils::FileType;

use crate::errors::Error;

pub(crate) trait DataProvider {
    fn get_name(&self) -> String;
    fn get_path(&self) -> PathBuf;
    fn get_format(&self) -> FileType;
    fn get_size(&self) -> u64;
}

pub(crate) struct FileProvider {
    path: PathBuf,
    name: String,
    format: FileType,
    size: u64,
}

impl DataProvider for FileProvider {
    #[inline]
    fn get_name(&self) -> String {
        self.name.clone()
    }

    #[inline]
    fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    #[inline]
    fn get_format(&self) -> FileType {
        self.format.clone()
    }

    #[inline]
    fn get_size(&self) -> u64 {
        self.size
    }
}

impl FileProvider {
    pub(crate) fn new(path: PathBuf) -> Result<Self, Error> {
        let path = path.clone();
        let mut file = File::open(path.clone())?;
        let format = FileType::parse(&mut file)?;
        let size = file.metadata().map_err(|_| Error::Metadata)?.len();
        let name = path
            .file_name()
            .ok_or(Error::Path)?
            .to_str()
            .ok_or(Error::Path)?
            .to_string();
        Ok(FileProvider {
            name,
            format,
            path,
            size,
        })
    }
}
