extern crate ncmdump;
extern crate serde_json;
extern crate structopt;

use std::error::Error;
use std::fs::{read, write};
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

use ncmdump::{decode, get_info};

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

fn get_output(file_path: &PathBuf, format: &str, output: &Option<PathBuf>) -> PathBuf {
    let parent = match output {
        None => file_path.parent().unwrap(),
        Some(p) => p,
    };
    let file_name = file_path.file_stem().unwrap();
    parent.join(file_name).with_extension(format)
}

fn run(options: Opt) -> Result<(), Box<dyn Error>> {
    let Opt {
        files,
        output,
        verbose,
        info,
    } = options;

    for file in files {
        if verbose {
            let file_name = file.file_name().unwrap().to_str().unwrap();
            println!("{}", file_name);
        }
        let buffer = read(&file)?;
        let modify = get_info(&buffer)?;
        if info {
            println!("{}", serde_json::to_string_pretty(&modify)?);
            exit(0);
        }
        let output_file = get_output(&file, &modify.format, &output);
        println!("{:?}", output_file);
        let data = decode(&buffer)?;
        write(output_file, data)?;
    }
    Ok(())
}

fn main() {
    let options = Opt::from_args();
    run(options).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        exit(1);
    });
}
