//! This example is use `NcmDump` to convert a ncm file to flac file
//!
//! You should use your own qmcflac file instead the test file
//!
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use ncmdump::Ncmdump;

fn main() -> Result<()> {
    let file = File::open("tests/test.ncm")?;
    let mut ncm = Ncmdump::from_reader(file)?;
    let data = ncm.get_data()?;

    let mut target = File::options()
        .create(true)
        .write(true)
        .open("tests/test.flac")?;
    target.write_all(&data)?;
    Ok(())
}
