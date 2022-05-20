use std::fs::{read, write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use structopt::StructOpt;

use ncmdump::{convert, get_info};

#[derive(Debug, StructOpt)]
#[structopt(name = "ncmdump")]
struct Opt {
    /// Specified the files
    #[structopt(short = "f", long = "files", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Specified the output directory
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: Option<PathBuf>,

    /// Verbosely list files processing
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Only show the information of files
    #[structopt(short = "i", long = "info")]
    info: bool,
}

fn get_output(file_path: &Path, format: &str, output: &Option<PathBuf>) -> Result<PathBuf> {
    let parent = match output {
        None => file_path.parent().unwrap(),
        Some(p) => p,
    };
    let file_name = file_path.file_stem().unwrap();
    let path = parent.join(file_name).with_extension(format);
    Ok(path)
}

fn run(options: Opt) -> Result<()> {
    let Opt {
        files,
        output,
        verbose,
        info,
    } = options;

    for file in files {
        if verbose {
            let file_name = file.file_name().unwrap().to_str().unwrap();
            print!("Converting file {}", file_name);
        }
        let buffer = read(&file)?;
        let modify = get_info(&buffer)?;
        if info {
            println!("{}", serde_json::to_string_pretty(&modify)?);
            continue;
        }
        let output_file = get_output(&file, &modify.format, &output)?;
        let data = convert(&buffer)?;
        write(output_file, data)?;
        if verbose {
            println!("\tcomplete!")
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let options = Opt::from_args();
    run(options)?;
    Ok(())
}
