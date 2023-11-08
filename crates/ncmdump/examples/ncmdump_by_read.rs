//! This example is use `Ncmdump` to convert a ncm file to flac file.
//! But this example is use the `read` method to read file.
//! You can use the same method to read qmc file.
//!
//! You should use your own qmcflac file instead the test file
//!
use std::fs::File;
use std::io::{Error, Read, Write};

use ncmdump::Ncmdump;

fn main() -> Result<(), Error> {
    let file = File::open("res/test.ncm")?;
    let mut ncm = Ncmdump::from_reader(file).expect("Can't create dump");
    let mut buf = [0; 1024];
    let mut target = File::options()
        .create(true)
        .write(true)
        .open("res/test.flac")?;

    while let Ok(size) = ncm.read(&mut buf) {
        if size == 0 {
            break;
        }
        target.write_all(&buf[..size])?;
    }
    Ok(())
}
