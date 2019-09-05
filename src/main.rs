extern crate ncmdump;
extern crate structopt;

use std::fs::{read, write};
use std::io::Result;
use std::path::PathBuf;
use structopt::StructOpt;

use ncmdump::tools::{Modify};
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

fn main() -> Result<()> {
    let Opt { files, verbose } = Opt::from_args();

    for file in files {
        if verbose {
            println!("{}", file.file_name().unwrap().to_str().unwrap());
        }
        let buffer = read(&file)?;
        let Modify { format, .. } = match get_info(&buffer) {
            Ok(i) => i,
            Err(err) => panic!("Error: {}", err),
        };
        let mut output_file = PathBuf::from(&file);
        output_file.set_extension(format);
        match decode(&buffer) {
            Ok(data) => write(output_file, data)?,
            Err(err) => panic!("Error: {}", err),
        };
    }
    Ok(())
}
