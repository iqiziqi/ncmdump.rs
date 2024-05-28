//! This example is use `QmcDump` to convert a qmc file to flac file
//! You can use the same method to read ncm file. Like [this example](examples/ncmdump.rs)
//!
//! You should use your own qmcflac file instead the test file
//!
use std::fs::File;
use std::io::{Error, Write};

use ncmdump::QmcDump;

fn main() -> Result<(), Error> {
    let file = File::open("res/test.qmcflac")?;
    let mut qmc = QmcDump::from_reader(file).expect("Can't create dump");
    let data = qmc.get_data()?;

    let mut target = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open("res/test.flac")?;
    target.write_all(&data)?;
    Ok(())
}
