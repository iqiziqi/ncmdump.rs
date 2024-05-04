use crate::error::{Errors, Result};

#[cfg(feature = "id3")]
pub trait Tag {
    fn get_tag(&mut self) -> Result<id3::Tag> {
        Err(Errors::InvalidFileType)
    }
}
