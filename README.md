# Ncmdump.rs

![](https://github.com/iqiziqi/ncmdump.rs/workflows/test/badge.svg)
![](https://github.com/iqiziqi/ncmdump.rs/workflows/deploy/badge.svg)
![](https://img.shields.io/crates/v/ncmdump)
![](https://img.shields.io/crates/d/ncmdump)

## Credit

### Origin

* [anonymous5l/ncmdump](https://github.com/anonymous5l/ncmdump): Original repository
* [MegrezZhu/qmcdump](https://github.com/MegrezZhu/qmcdump): Original repository

## Binary Usage

### Install

You can install by Cargo.

```shell
cargo install ncmdump
```

### Simple usage

```shell
ncmdump [FILES]...
```

### More options

```
Usage: ncmdump [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Specified the files to convert

Options:
  -o, --output <OUTPUT>  Specified the output directory. Default it's the same directory with input file
  -v, --verbose          Verbosely list files processing
  -i, --info             Only show the ncm information of ncm files. If not ncm file, will print empty object
  -h, --help             Print help
  -V, --version          Print version
```

## Library Usage

### Install

You can add this to your `Cargo.toml`:

```toml
ncmdump = "0.5.0"
```

Also, you can use this command to install this crate,
if you installed [cargo-edit](https://github.com/killercup/cargo-edit)

```shell
cargo add ncmdump
```

### Simple Usage

```rust
use std::fs::File;
use std::path::Path;

use anyhow::Result;
use ncmdump::Ncmdump;

fn main() -> Result<()> {
    use std::io::Write;
    let file = File::open("tests/test.ncm")?;
    let mut ncm = Ncmdump::from_reader(file)?;
    let music = ncm.get_data()?;
    let mut target = File::options()
        .create(true)
        .write(true)
        .open("tests/test.flac")?;
    target.write_all(&music)?;
    Ok(())
}
```

For more usage, please check the [document](https://docs.rs/ncmdump).
