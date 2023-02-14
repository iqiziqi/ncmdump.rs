use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use thiserror::Error;

use ncmdump::{Ncmdump, QmcDump};

#[derive(Clone, Debug, Error)]
enum Error {
    #[error("Can't resolve the path")]
    PathError,
    #[error("Invalid file format")]
    FormatError,
}

struct Wrapper {
    path: PathBuf,
    size: u64,
    name: String,
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

fn get_paths(matchers: &Vec<String>) -> Result<Vec<Wrapper>> {
    let mut paths = Vec::new();
    for matcher in matchers {
        for entry in glob(matcher)? {
            let path = entry?;
            let file = File::open(&path)?;
            let size = file.metadata()?.len();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            paths.push(Wrapper { path, size, name })
        }
    }
    Ok(paths)
}

fn main() -> Result<()> {
    let command = Command::parse();

    let list = get_paths(&command.matchers)?;
    let total_size = list.iter().map(|item| item.size).sum();
    let template = "{bytes:>10!}/{total_bytes:<10!} {bytes_per_sec:>15!} [{bar:40}]";
    let progress_style = ProgressStyle::with_template(template)?;
    let progress = ProgressBar::new(total_size).with_style(progress_style);
    for item in list {
        let file_type = check_format(&item.path)?;
        let mut data = Vec::new();
        match file_type {
            FileType::Ncm => {
                let file = File::open(&item.path)?;
                let mut dump = Ncmdump::from_reader(file)?;
                let mut buffer = [0; 1024];
                while let Ok(size) = dump.read(&mut buffer) {
                    if size == 0 {
                        break;
                    }
                    data.write_all(&buffer[..size])?;
                    progress.inc(size as u64);
                }
            }
            FileType::Qmc => {
                let file = File::open(&item.path)?;
                let mut dump = QmcDump::from_reader(file)?;
                let mut buffer = [0; 1024];
                while let Ok(size) = dump.read(&mut buffer) {
                    if size == 0 {
                        break;
                    }
                    data.write_all(&buffer[..size])?;
                    progress.inc(size as u64);
                }
            }
        }
        let ext = match data[..4] {
            [0x66, 0x4C, 0x61, 0x43] => Ok("flac"),
            [0x49, 0x44, 0x33, _] => Ok("mp3"),
            _ => Err(Error::FormatError),
        }?;
        let output_file = get_output(&item.path, ext, &command.output)?;
        let mut target = File::options().create(true).write(true).open(output_file)?;
        target.write_all(&data)?;
        if command.verbose {
            progress.println(format!("Converting file {}\t complete!", item.name));
        }
    }
    progress.finish();
    Ok(())
}
