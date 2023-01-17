use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use structopt::StructOpt;

use ncmdump::Ncmdump;

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

fn main() -> Result<()> {
    let options = Opt::from_args();
    let Opt {
        files,
        output,
        verbose,
        info,
    } = options;

    for file_path in files {
        if verbose {
            let file_name = file_path.file_name().unwrap().to_str().unwrap();
            print!("Converting file {}", file_name);
        }
        let file_ = File::open(&file_path)?;
        let mut ncm = Ncmdump::from_reader(file_)?;
        let information = ncm.get_info()?;
        if info {
            println!("{}", serde_json::to_string_pretty(&information)?);
            continue;
        }
        let output_file = get_output(&file_path, &information.format, &output)?;
        let mut output = File::options().create(true).write(true).open(output_file)?;
        let data = ncm.get_data()?;
        output.write_all(&data)?;

        if verbose {
            println!("\tcomplete!")
        }
    }
    Ok(())
}
