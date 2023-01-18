use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use glob::glob;
use thiserror::Error;

use ncmdump::Ncmdump;

#[derive(Clone, Debug, Error)]
#[error("Can't resolve the path")]
pub struct PathError;

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
    #[arg(short = 'i', long = "info")]
    info: bool,
}

fn get_output(file_path: &Path, format: &str, output: &Option<String>) -> Result<PathBuf> {
    let parent = match output {
        None => file_path.parent().ok_or(PathError)?,
        Some(p) => Path::new(p),
    };
    let file_name = file_path.file_stem().ok_or(PathError)?;
    let path = parent.join(file_name).with_extension(format);
    Ok(path)
}

fn main() -> Result<()> {
    let Command {
        matchers,
        output,
        verbose,
        info,
    } = Command::parse();

    for matcher in matchers {
        for entry in glob(&matcher)? {
            let entry = entry?;
            let file = File::open(&entry)?;
            let mut ncm = Ncmdump::from_reader(file)?;
            let information = ncm.get_info()?;
            if info {
                println!("{}", serde_json::to_string_pretty(&information)?);
                continue;
            }
            let output_file = get_output(&entry, &information.format, &output)?;
            let mut output = File::options().create(true).write(true).open(output_file)?;
            let data = ncm.get_data()?;
            output.write_all(&data)?;

            if verbose {
                let file_name = entry
                    .file_name()
                    .ok_or(PathError)?
                    .to_str()
                    .ok_or(PathError)?;
                print!("Converting file {file_name}\t complete!");
            }
        }
    }
    Ok(())
}
