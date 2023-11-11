use anyhow::Result;
use id3::frame::Picture;
use id3::TagLike;
use ncmdump::NcmInfo;

use crate::utils::get_image_mime_type;

trait Metadata {
    fn write_metadata(&self) -> Result<()>;
}

struct Mp3Metadata<'a> {
    info: &'a NcmInfo,
    image: &'a [u8],
}

impl<'a> Metadata for Mp3Metadata<'a> {
    fn write_metadata(&self) -> Result<()> {
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
        Ok(())
    }
}

struct FlacMetadata<'a> {
    info: &'a NcmInfo,
    image: &'a [u8],
}

impl<'a> Metadata for FlacMetadata<'a> {
    fn write_metadata(&self) -> Result<()> {
        let mut tag = metaflac::Tag::new();
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
        Ok(())
    }
}
