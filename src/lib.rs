extern crate aes_soft;
extern crate base64;
extern crate block_modes;
extern crate serde;

mod error;

use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use aes_soft::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Ecb};
use serde::Deserialize;

pub use error::{Error, ErrorKind};

type Aes128Ecb = Ecb<Aes128, Pkcs7>;

static HEADER_KEY: [u8; 16] = [
    0x68, 0x7A, 0x48, 0x52, 0x41, 0x6D, 0x73, 0x6F,
    0x35, 0x6B, 0x49, 0x6E, 0x62, 0x61, 0x78, 0x57,
];

static MODIFY_KEY: [u8; 16] = [
    0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21,
    0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C, 0x27, 0x28,
];

#[derive(Debug, Deserialize)]
struct Modify {
    #[serde(rename = "musicName")]
    name: String,
    #[serde(rename = "musicId")]
    id: u64,
    album: String,
    alias: Vec<String>,
    artist: Vec<(String, u64)>,
    bitrate: u64,
    duration: u64,
    format: String,
}

pub fn process(input: &Path) -> Result<(), Error> {
    let file = File::open(input).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => Error::from(ErrorKind::FileNotFound),
        io::ErrorKind::PermissionDenied => Error::from(ErrorKind::PermissionDenied),
        _ => Error::from(ErrorKind::Unknown),
    })?;
    let mut reader = BufReader::new(&file);

    check_format(&mut reader)?;
    let key = get_key(&mut reader)?;
    let modify = get_modify(&mut reader)?;
    crc_check(&mut reader)?;
    get_image(&mut reader)?;

    let key_box = build_key_box(&key);
    let output_path = make_output_path(input, &modify);
    let output_file = File::create(output_path).map_err(|_| Error::from(ErrorKind::ReadOrWrite))?;
    let mut writer = BufWriter::new(&output_file);
    write_file(&mut reader, &mut writer, &key_box).map_err(|err| match err.kind() {
        io::ErrorKind::PermissionDenied => Error::from(ErrorKind::PermissionDenied),
        _ => Error::from(ErrorKind::ReadOrWrite),
    })?;

    Ok(())
}

fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    let cipher = Aes128Ecb::new_var(&key, &[]).unwrap();
    let a = cipher.decrypt_vec(data).unwrap();
    Ok(a)
}

fn check_format(reader: &mut BufReader<&File>) -> Result<(), Error> {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer)
        .map_err(|_| Error::from(ErrorKind::ReadOrWrite))?;
    if u64::from_ne_bytes(buffer) != 0x4d41_4446_4e45_5443 {
        return Err(Error::from(ErrorKind::InvalidFile));
    }
    reader.read_exact(&mut [0u8; 2])
        .map_err(|_| Error::from(ErrorKind::ReadOrWrite))?;
    Ok(())
}

fn get_length(reader: &mut BufReader<&File>) -> Result<u32, Error> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)
        .map_err(|_| Error::from(ErrorKind::ReadOrWrite))?;

    Ok(u32::from_ne_bytes(buffer))
}

fn get_key(reader: &mut BufReader<&File>) -> Result<Vec<u8>, Error> {
    let length = get_length(reader)?;
    let mut buffer = vec![0u8; length as usize];
    reader.read_exact(&mut buffer)
        .map_err(|_| Error::from(ErrorKind::ReadOrWrite))?;

    let key_buffer = buffer.iter().map(|byte| byte ^ 0x64).collect::<Vec<u8>>();
    let decrypt_buffer = decrypt(&key_buffer, &HEADER_KEY)?;
    Ok(decrypt_buffer[17..].to_vec())
}

fn get_modify(reader: &mut BufReader<&File>) -> Result<Modify, Error> {
    let length = get_length(reader)?;
    let mut buffer = vec![0u8; length as usize];
    reader.read_exact(&mut buffer)
        .map_err(|_| Error::from(ErrorKind::ReadOrWrite))?;

    let modify_tmp = buffer.iter().map(|item| item ^ 0x63).collect::<Vec<u8>>();
    let modify_key = base64::decode(&modify_tmp[22..])
        .map_err(|_| Error::from(ErrorKind::InvalidFile))?;
    let modify_str = String::from_utf8(decrypt(&modify_key, &MODIFY_KEY)?[6..].to_vec())
        .map_err(|_| Error::from(ErrorKind::InvalidFile))?;
    let modify =serde_json::from_str::<Modify>(&modify_str)
        .map_err(|_| Error::from(ErrorKind::InvalidFile))?;

    Ok(modify)
}

fn crc_check(reader: &mut BufReader<&File>) -> Result<(), Error> {
    let mut buffer = [0u8; 9];
    reader.read_exact(&mut buffer)
        .map_err(|_| Error::from(ErrorKind::ReadOrWrite))?;
    Ok(())
}

fn get_image(reader: &mut BufReader<&File>) -> Result<Vec<u8>, Error> {
    let length = get_length(reader)?;
    let mut buffer = vec![0u8; length as usize];
    reader.read_exact(&mut buffer)
        .map_err(|_| Error::from(ErrorKind::ReadOrWrite))?;
    Ok(buffer)
}

fn make_output_path(input_path: &Path, modify: &Modify) -> PathBuf {
    let Modify { format, name, artist, .. } = modify;
    let default_output_name = || format!("{} - {}", artist[0].0, name);
    let output_name = input_path
        .file_stem()
        .map_or_else(default_output_name, |stem| {
            stem.to_str().map_or_else(default_output_name, String::from)
        });
    let output_file = format!("{}.{}", output_name, format);
    input_path.parent().unwrap().join(output_file)
}

fn write_file(
    reader: &mut BufReader<&File>,
    writer: &mut BufWriter<&File>,
    key_box: &[usize],
) -> Result<(), io::Error> {
    let mut write_buffer = [0u8; 0x8000];
    while reader.read(&mut write_buffer)? > 0 {
        let buffer = write_buffer.iter().enumerate().map(|(index, item)| {
            let j = (index + 1) & 0xff;
            item ^ key_box[(key_box[j] + key_box[(key_box[j] + j) & 0xff]) & 0xff] as u8
        })
        .collect::<Vec<u8>>();
        writer.write_all(&buffer)?;
    }
    writer.flush()?;

    Ok(())
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
