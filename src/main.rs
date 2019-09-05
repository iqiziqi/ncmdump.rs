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

    /// This option can specified the files.
    #[structopt(short = "f", long = "files", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let Opt { files } = Opt::from_args();

    for file in files {
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
