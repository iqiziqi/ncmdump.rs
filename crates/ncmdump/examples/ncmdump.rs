//! This example is use `NcmDump` to convert a ncm file to flac file
//! You can use the same method to read qmc file. Like [this example](examples/qcmdump.rs)
//!
//! You should use your own qmcflac file instead the test file
//!
use std::fs::File;
use std::io::{Error, Write};

use ncmdump::Ncmdump;

fn main() -> Result<(), Error> {
    let file = File::open("res/test.ncm")?;
    let mut ncm = Ncmdump::from_reader(file).expect("Can't create dump");
    let data = ncm.get_data().expect("Can't get data");

    let mut target = File::options()
        .create(true)
        .write(true)
        .open("res/test.flac")?;
    target.write_all(&data)?;
    Ok(())
}
