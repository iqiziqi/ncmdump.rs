extern crate ncmdump;
extern crate structopt;

use std::error::Error;
use std::fs::{read, write};
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

use ncmdump::utils::Modify;
use ncmdump::{decode, get_info};

#[derive(Debug, StructOpt)]
#[structopt(name = "ncmdump")]
struct Opt {
    /// Specified the files
    #[structopt(short = "f", long = "files", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Verbosely list files processing
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}

fn run(options: Opt) -> Result<(), Box<dyn Error>> {
    let Opt { files, verbose } = options;
    for file in files {
        if verbose {
            println!("{}", file.file_name().unwrap().to_str().unwrap());
        }
        let buffer = read(&file)?;
        let Modify { format, .. } = get_info(&buffer)?;
        let mut output_file = PathBuf::from(&file);
        output_file.set_extension(format);
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
