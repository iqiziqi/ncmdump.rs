# Ncmdump.rs

![](https://github.com/iqiziqi/ncmdump.rs/workflows/test/badge.svg)
![](https://github.com/iqiziqi/ncmdump.rs/workflows/deploy/badge.svg)
[![](https://img.shields.io/crates/v/ncmdump)](https://crates.io/crates/ncmdump)
[![](https://img.shields.io/crates/d/ncmdump)](https://crates.io/crates/ncmdump)

[[English](./README.md)] [[中文](./README.cn)]

## 致谢

### 原始仓库

* [anonymous5l/ncmdump](https://github.com/anonymous5l/ncmdump)
* [MegrezZhu/qmcdump](https://github.com/MegrezZhu/qmcdump)

## 使用方法（二进制文件使用）

### 安装

通过 Cargo 工具进行安装。

```shell
cargo install ncmdump
```

### 简单使用

```shell
ncmdump [FILES]...
```

### 更多选项

```text
Usage: ncmdump [OPTIONS] [FILES]...

Arguments:
  [FILES]...  Specified the files to convert

Options:
  -o, --output <OUTPUT>  Specified the output directory. Default it's the same directory with input file
  -v, --verbose          Verbosely list files processing
  -h, --help             Print help
  -V, --version          Print version
```

## 使用方法（作为库使用）

### 安装

在你的 `Cargo.toml` 文件中添加依赖：

```toml
ncmdump = "0.7.3"
```

或者，如果安装了 [cargo-edit](https://github.com/killercup/cargo-edit)，你可以使用命令：

```shell
cargo add ncmdump
```

### 简单用法

```rust
use std::fs::File;
use std::path::Path;

use anyhow::Result;
use ncmdump::Ncmdump;

fn main() -> Result<()> {
    use std::io::Write;
    let file = File::open("res/test.ncm")?;
    let mut ncm = Ncmdump::from_reader(file)?;
    let music = ncm.get_data()?;
    let mut target = File::options()
        .create(true)
        .write(true)
        .open("res/test.flac")?;
    target.write_all(&music)?;
    Ok(())
}
```

更多的用法, 可以查看[文档](https://docs.rs/ncmdump).
