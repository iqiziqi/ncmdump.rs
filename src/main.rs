use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use clap::{command, Parser};
use glob::glob;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use thiserror::Error;

#[cfg(feature = "utils")]
use ncmdump::utils::{get_file_type, FileType};
#[cfg(feature = "ncmdump")]
use ncmdump::Ncmdump;
#[cfg(feature = "qmcdump")]
use ncmdump::QmcDump;

const PROGRESS_STYLE_DUMP: &str =
    "[{bar:40.cyan}] |{percent:>3!}%| {bytes:>7}/{total_bytes:7} {msg}";
const TOTAL_STYLE_DUMP: &str = "[{bar:40.cyan}] |{percent:>3!}%| {bytes:>7}/{total_bytes:7}";

#[derive(Clone, Debug, Error)]
enum Error {
    #[error("Can't resolve the path")]
    Path,
    #[error("Invalid file format")]
    Format,
    #[error("No file can be converted")]
    NoFile,
    #[error("Can't get file's metadata")]
    Metadata,
    #[error("Worker can't less than 0 and more than 8")]
    Worker,
}

#[derive(Clone, Debug, Default, Parser)]
#[command(name = "ncmdump", bin_name = "ncmdump", about, version)]
pub struct Command {
    /// Specified the files to convert.
    #[arg(value_name = "FILES")]
    matchers: Vec<String>,

    /// Specified the output directory.
    /// Default it's the same directory with input file.
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Verbosely list files processing.
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// The process work count.
    /// It should more than 0 and less than 9.
    #[arg(short = 'w', long = "worker", default_value = "1")]
    worker: usize,
}

impl Command {
    fn get_output(file_path: &Path, format: &str, output: &Option<String>) -> Result<PathBuf> {
        let parent = match output {
            None => file_path.parent().ok_or(Error::Path)?,
            Some(p) => Path::new(p),
        };
        let file_name = file_path.file_name().ok_or(Error::Path)?;
        let path = parent.join(file_name).with_extension(format);
        Ok(path)
    }

    fn get_data(
        mut dump: impl Read,
        total: &ProgressBar,
        progress: &Option<ProgressBar>,
    ) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        let mut buffer = [0; 1024];
        while let Ok(size) = dump.read(&mut buffer) {
            if size == 0 {
                break;
            }
            data.write_all(&buffer[..size])?;
            total.inc(size as u64);
            if let Some(p) = progress {
                p.inc(size as u64);
            }
        }
        if let Some(p) = progress {
            p.finish();
        }
        Ok(data)
    }

    fn dump(
        &self,
        item: &Wrapper,
        total: &ProgressBar,
        progress: &Option<ProgressBar>,
    ) -> Result<()> {
        let file = File::open(&item.path)?;
        let data = match item.format {
            #[cfg(feature = "ncmdump")]
            FileType::Ncm => Self::get_data(Ncmdump::from_reader(file)?, total, progress),
            #[cfg(feature = "qmcdump")]
            FileType::Qmc => Self::get_data(QmcDump::from_reader(file)?, total, progress),
            FileType::Other => Err(Error::Format.into()),
        }?;
        let ext = match data[..4] {
            [0x66, 0x4C, 0x61, 0x43] => Ok("flac"),
            [0x49, 0x44, 0x33, _] => Ok("mp3"),
            _ => Err(Error::Format),
        }?;
        let output_file = Self::get_output(&item.path, ext, &self.output)?;
        let mut target = File::options().create(true).write(true).open(output_file)?;
        target.write_all(&data)?;
        Ok(())
    }
}

struct Wrapper {
    name: String,
    format: FileType,
    path: PathBuf,
    size: u64,
}

impl Wrapper {
    fn from_path(path: PathBuf) -> Result<Self> {
        let mut file = File::open(&path)?;
        let format = get_file_type(&mut file)?;
        let size = file.metadata().map_err(|_| Error::Metadata)?.len();
        let name = path
            .file_name()
            .ok_or(Error::Path)?
            .to_str()
            .ok_or(Error::Path)?
            .to_string();
        Ok(Self {
            name,
            format,
            path,
            size,
        })
    }
}

struct NcmdumpCli(Arc<Command>);

impl NcmdumpCli {
    fn from_command(command: Command) -> Self {
        Self(Arc::new(command))
    }

    fn start(&self) -> Result<()> {
        let worker = match self.0.worker {
            1..=8 => Ok(self.0.worker),
            _ => Err(Error::Worker),
        }?;

        let matchers = self.0.matchers.clone();
        if matchers.is_empty() {
            return Err(Error::NoFile.into());
        }
        let mut tasks = Vec::new();
        let progress = Arc::new(MultiProgress::new());
        let total_progress_style = ProgressStyle::with_template(TOTAL_STYLE_DUMP)?;
        let total = Arc::new(progress.add(ProgressBar::new(0).with_style(total_progress_style)));
        let (tx, rx) = crossbeam_channel::unbounded();

        {
            let total = total.clone();
            thread::spawn(move || {
                for matcher in matchers {
                    for entry in glob(matcher.as_str())? {
                        let path = entry.map_err(|_| Error::Path)?;
                        if path.is_file() {
                            let w = Wrapper::from_path(path).map_err(|_| Error::Path)?;
                            total.set_length(total.length().unwrap_or(0) + w.size);
                            tx.send(w)?;
                        }
                    }
                }
                anyhow::Ok(())
            });
        }

        for _ in 1..=worker {
            let rx = rx.clone();
            let total = total.clone();
            let progress = progress.clone();
            let command = self.0.clone();
            let task = thread::spawn(move || {
                let progress_style_dump = ProgressStyle::with_template(PROGRESS_STYLE_DUMP)?;
                while let Ok(w) = rx.recv() {
                    let current = if command.verbose {
                        let current = progress.insert_from_back(
                            1,
                            ProgressBar::new(w.size).with_style(progress_style_dump.clone()),
                        );
                        current.set_message(w.name.clone());
                        Some(current)
                    } else {
                        None
                    };
                    command.dump(&w, &total, &current)?;
                }
                anyhow::Ok(())
            });
            tasks.push(task);
        }
        for task in tasks {
            task.join().unwrap()?;
        }
        total.finish();
        Ok(())
    }
}

fn main() -> Result<()> {
    NcmdumpCli::from_command(Command::parse()).start()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{Command, NcmdumpCli};

    #[test]
    fn test_empty_input_files_err() -> Result<()> {
        let command = Command {
            matchers: vec![],
            worker: 1,
            ..Default::default()
        };
        let result = NcmdumpCli::from_command(command).start();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_invalid_worker_err() -> Result<()> {
        let works = [0, 9, 10, 15, 100, 199];
        for worker in works {
            let command = Command {
                matchers: vec![],
                worker,
                ..Default::default()
            };
            let result = NcmdumpCli::from_command(command).start();
            assert!(result.is_err());
        }
        Ok(())
    }

    #[test]
    fn test_worker_ok() -> Result<()> {
        for worker in 1..=8 {
            let command = Command {
                matchers: vec!["./test/test.ncm".into()],
                worker,
                ..Default::default()
            };
            let result = NcmdumpCli::from_command(command).start();
            assert!(result.is_ok());
        }
        Ok(())
    }
}
