use std::io::{Cursor, Write};

use anyhow::Result;
use id3::{TagLike, Version};
use id3::frame::Picture;

use ncmdump::NcmInfo;

use crate::utils::get_image_mime_type;

pub(crate) trait Metadata {
    fn write_metadata(&self, data: Vec<u8>) -> Result<Vec<u8>>;
}

pub(crate) struct Mp3Metadata<'a> {
    pub(crate) info: &'a NcmInfo,
    pub(crate) image: &'a [u8],
}

impl<'a> Metadata for Mp3Metadata<'a> {
    fn write_metadata(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(data.clone());
        let mut tag = id3::Tag::new();
        let artist = &self
            .info
            .artist
            .iter()
            .map(|item| item.0.to_owned())
            .collect::<Vec<String>>()
            .join("/");
        tag.set_title(&self.info.name);
        tag.set_album(&self.info.album);
        tag.set_artist(artist);

        if !self.image.is_empty() {
            tag.add_frame(Picture {
                mime_type: get_image_mime_type(self.image).to_owned(),
                picture_type: id3::frame::PictureType::CoverFront,
                description: "".to_string(),
                data: self.image.to_vec(),
            });
        }
        tag.write_to(&mut cursor, Version::Id3v24)?;
        Ok(cursor.into_inner())
    }
}

pub(crate) struct FlacMetadata<'a> {
    pub(crate) info: &'a NcmInfo,
    pub(crate) image: &'a [u8],
}

impl<'a> Metadata for FlacMetadata<'a> {
    fn write_metadata(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        let mut tag = metaflac::Tag::read_from(&mut Cursor::new(&data))?;
        let data = metaflac::Tag::skip_metadata(&mut Cursor::new(&data));
        let mc = tag.vorbis_comments_mut();
        let artist = self
            .info
            .artist
            .iter()
            .cloned()
            .map(|item| item.0)
            .collect::<Vec<String>>();
        mc.set_title(vec![self.info.name.to_string()]);
        mc.set_album(vec![self.info.album.to_string()]);
        mc.set_artist(artist);
        tag.add_picture(
            get_image_mime_type(self.image),
            metaflac::block::PictureType::CoverFront,
            self.image.to_vec(),
        );
        tag.remove_blocks(metaflac::BlockType::Padding);
        let mut buffer = Vec::new();
        tag.write_to(&mut buffer)?;
        buffer.write_all(&data)?;
        Ok(buffer)
    }
}
