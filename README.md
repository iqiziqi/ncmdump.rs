# Ncmdump.rs

## Credit

### Origin

* [anonymous5l/ncmdump](https://github.com/anonymous5l/ncmdump): Original repository

## Binary Usage

### Simple usage

```shell
ncmdump -f [files ...]
```

### More options

```
USAGE:
    ncmdump [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -i, --info       Only show the information of files
    -V, --version    Prints version information
    -v, --verbose    Verbosely list files processing

OPTIONS:
    -f, --files <files>...    Specified the files
    -o, --output <output>     Specified the output directory
```

## Library Usage

### Install

You can add this to your `Cargo.toml`:

```toml
ncmdump = "0.1.0"
```

Also, you can use this command to install this crate,
if you installed [cargo-edit](https://github.com/killercup/cargo-edit)

```shell
cargo add ncmdump
```

### Simple Usage

```rust
extern crate ncmdump;

use std::error::Error;
use std::fs::{read, write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("tests/test.ncm");
    let output_path = Path::new("tests/test.flac");
    let buffer = read(&input_path)?;
    let data = ncmdump::convert(&buffer)?;
    write(&output_path, data)?;
    Ok(())
}
```
