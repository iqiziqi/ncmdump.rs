extern crate ncmdump;
extern crate structopt;

use ncmdump::process;
use structopt::StructOpt;

use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(name = "ncmdump")]
struct Opt {
    /// This option can specified the files.
    #[structopt(short = "f", long = "files", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Use auto file name instead of input file name.
    #[structopt(short = "a", long = "auto-name")]
    auto: bool,
}

fn main() {
    let Opt { files, .. } = Opt::from_args();
    for file in files {
        if let Err(err) = process(&file) {
            println!("Error: {}", err);
        }
    }
}
