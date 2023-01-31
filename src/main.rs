use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use glob::glob;
use thiserror::Error;

use ncmdump::{Ncmdump, QmcDump};

#[derive(Clone, Debug, Error)]
enum Error {
    #[error("Can't resolve the path")]
    PathError,
    #[error("Invalid file format")]
    FormatError,
}

enum FileType {
    Ncm,
    Qmc,
}

#[derive(Debug, Parser)]
#[command(name = "ncmdump", bin_name = "ncmdump", about, version)]
struct Command {
    /// Specified the files to convert.
    #[arg(value_name = "FILES")]
    matchers: Vec<String>,

    /// Specified the output directory.
    /// Default it's the same directory with input file.
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Verbosely list files processing.
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Only show the ncm information of ncm files.
    /// If not ncm file, will print empty object.
    #[arg(short = 'i', long = "info")]
    info: bool,
}

fn check_format(path: &PathBuf) -> Result<FileType> {
    let mut file = File::open(path)?;
    let mut head = [0; 8];
    let count = file.read(&mut head)?;
    if count != 8 {
        return Err(Error::FormatError.into());
    }
    let file_type = match head[..] {
        [0x43, 0x54, 0x45, 0x4E, 0x46, 0x44, 0x41, 0x4D] => FileType::Ncm,
        [0xA5, 0x06, 0xB7, 0x89, _, _, _, _] => FileType::Qmc,
        [0x8A, 0x0E, 0xE5, _, _, _, _, _] => FileType::Qmc,
        _ => return Err(Error::FormatError.into()),
    };
    Ok(file_type)
}

fn get_output(file_path: &Path, format: &str, output: &Option<String>) -> Result<PathBuf> {
    let parent = match output {
        None => file_path.parent().ok_or(Error::PathError)?,
        Some(p) => Path::new(p),
    };
    let file_name = file_path.file_stem().ok_or(Error::PathError)?;
    let path = parent.join(file_name).with_extension(format);
    Ok(path)
}

fn get_paths(matchers: Vec<String>) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for matcher in matchers {
        for entry in glob(&matcher)? {
            paths.push(entry?);
        }
    }
    Ok(paths)
}

fn main() -> Result<()> {
    let Command {
        matchers,
        output,
        verbose,
        info,
    } = Command::parse();

    for entry in get_paths(matchers)? {
        let file = File::open(&entry)?;
        let file_type = check_format(&entry)?;
        if info {
            let info = match file_type {
                FileType::Ncm => {
                    let mut ncm = Ncmdump::from_reader(file)?;
                    let information = ncm.get_info()?;
                    serde_json::to_string_pretty(&information)?
                }
                FileType::Qmc => "{}".to_string(),
            };
            println!("{}", info);
            continue;
        }
        let data = match file_type {
            FileType::Ncm => Ncmdump::from_reader(file)?.get_data()?,
            FileType::Qmc => QmcDump::from_reader(file)?.get_data()?,
        };
        let ext = match data[..4] {
            [0x66, 0x4C, 0x61, 0x43] => Ok("flac"),
            [0x49, 0x44, 0x33, _] => Ok("mp3"),
            _ => Err(Error::FormatError),
        }?;
        let output_file = get_output(&entry, ext, &output)?;
        let mut output = File::options().create(true).write(true).open(output_file)?;
        output.write_all(&data)?;

        if verbose {
            let file_name = entry
                .file_name()
                .ok_or(Error::PathError)?
                .to_str()
                .ok_or(Error::PathError)?;
            println!("Converting file {file_name}\t complete!");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::Error;
    use anyhow::Result;
    use glob::glob;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test() -> Result<()> {
        for path in glob("tests/*.ncm")? {
            let path = path?;
            let mut file = File::open(path)?;
            let mut head = [0; 8];
            if file.read(&mut head)? != 8 {
                return Err(Error::FormatError.into());
            }
            println!("header is {:X?}", head);
        }
        Ok(())
    }
}
