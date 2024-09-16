use std::io::{Cursor, Seek, SeekFrom, Write};

use anyhow::Result;
use id3::frame::Picture;
use id3::{TagLike, Version};

use ncmdump::NcmInfo;

use crate::utils::get_image_mime_type;

pub(crate) trait Metadata {
    /// Get the data with metadata.
    fn inject_metadata(&mut self, data: Vec<u8>) -> Result<Vec<u8>>;
}

pub(crate) struct Mp3Metadata(id3::Tag);

impl Mp3Metadata {
    pub(crate) fn new(info: &NcmInfo, image: &[u8], data: &[u8]) -> Self {
        let cursor = Cursor::new(data.to_vec());
        let mut tag = id3::Tag::read_from2(cursor).unwrap_or_else(|_| id3::Tag::new());
        let artist = info
            .artist
            .iter()
            .map(|item| item.0.to_owned())
            .collect::<Vec<String>>()
            .join("/");
        tag.set_title(&info.name);
        tag.set_album(&info.album);
        tag.set_artist(artist);
        if !image.is_empty() {
            tag.add_frame(Picture {
                mime_type: get_image_mime_type(image).to_owned(),
                picture_type: id3::frame::PictureType::CoverFront,
                description: "".to_string(),
                data: image.to_vec(),
            });
        }
        Self(tag)
    }
}

impl Metadata for Mp3Metadata {
    fn inject_metadata(&mut self, data: Vec<u8>) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(data);
        _ = cursor.seek(SeekFrom::Start(0));
        self.0.write_to_file(&mut cursor, Version::Id3v24)?;
        Ok(cursor.into_inner())
    }
}

pub(crate) struct FlacMetadata(metaflac::Tag);

impl FlacMetadata {
    pub(crate) fn new(info: &NcmInfo, image: &[u8], data: &[u8]) -> Self {
        let mut tag = metaflac::Tag::read_from(&mut Cursor::new(&data))
            .unwrap_or_else(|_| metaflac::Tag::new());
        let mc = tag.vorbis_comments_mut();
        let artist = info
            .artist
            .iter()
            .cloned()
            .map(|item| item.0)
            .collect::<Vec<String>>();
        mc.set_title(vec![info.name.to_string()]);
        mc.set_album(vec![info.album.to_string()]);
        mc.set_artist(artist);
        if !image.is_empty() {
            tag.add_picture(
                get_image_mime_type(image),
                metaflac::block::PictureType::CoverFront,
                image.to_vec(),
            );
        }
        Self(tag)
    }
}

impl Metadata for FlacMetadata {
    fn inject_metadata(&mut self, data: Vec<u8>) -> Result<Vec<u8>> {
        let data = metaflac::Tag::skip_metadata(&mut Cursor::new(&data));
        let mut buffer = Vec::new();
        self.0.remove_blocks(metaflac::BlockType::Padding);
        self.0.write_to(&mut buffer)?;
        buffer.write_all(&data)?;
        Ok(buffer)
    }
}
