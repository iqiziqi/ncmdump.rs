extern crate ncmdump;
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;
use ncmdump::process;

#[derive(Debug, StructOpt)]
#[structopt(name = "ncmdump")]
struct Opt {

    /// This option can specified the files.
    #[structopt(short = "f", long = "files", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let Opt { files } = Opt::from_args();
    for file in files {
        if let Err(err) = process(&file) {
            println!("Error: {}", err);
        }
    }
}
