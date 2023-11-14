use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use clap::Parser;
use errors::Error;
use glob::glob;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use ncmdump::utils::FileType;
use ncmdump::{Ncmdump, QmcDump};
use provider::{DataProvider, FileProvider};

mod command;
mod errors;
mod provider;

use crate::command::Command;

const TOTAL_PSTYPE: &str = "[{bar:40.cyan}] |{percent:>3!}%| {bytes:>10!}/{total_bytes:10!}";
const SINGLE_PSTYPE: &str = "[{bar:40.cyan}] |{percent:>3!}%| {bytes:>10!}/{total_bytes:10!} {msg}";

/// The global program
#[derive(Clone)]
struct Program {
    command: Arc<Command>,
    group: MultiProgress,
    total: ProgressBar,
}

impl Program {
    /// Create a new command progress.
    fn new(command: Command) -> Result<Self> {
        let group = MultiProgress::new();
        let style = ProgressStyle::with_template(TOTAL_PSTYPE)?;
        let total = group.add(ProgressBar::new(0).with_style(style));
        Ok(Self {
            command: Arc::new(command),
            group,
            total,
        })
    }

    /// Create a new progress.
    fn create_progress<P>(&self, provider: &P) -> Result<Option<ProgressBar>>
    where
        P: DataProvider,
    {
        if !self.command.verbose {
            return Ok(None);
        }
        let style = ProgressStyle::with_template(SINGLE_PSTYPE)?;
        let progress = self
            .group
            .insert_from_back(1, ProgressBar::new(provider.get_size()).with_style(style));
        progress.set_message(provider.get_name());
        Ok(Some(progress))
    }

    fn finish(&self) {
        self.total.finish();
    }

    fn dump<P>(&self, provider: &P) -> Result<()>
    where
        P: DataProvider,
    {
        let source = File::open(provider.get_path())?;
        let data = match provider.get_format() {
            FileType::Ncm => self.get_data(Ncmdump::from_reader(source)?, provider),
            FileType::Qmc => self.get_data(QmcDump::from_reader(source)?, provider),
            FileType::Other => Err(Error::Format.into()),
        }?;
        let ext = match data[..4] {
            [0x66, 0x4C, 0x61, 0x43] => Ok("flac"),
            [0x49, 0x44, 0x33, _] => Ok("mp3"),
            _ => Err(Error::Format),
        }?;
        let path = provider.get_path();
        let parent = match &self.command.output {
            None => path.parent().ok_or(Error::Path)?,
            Some(p) => Path::new(p),
        };
        let file_name = path.file_stem().ok_or(Error::Path)?;
        let path = parent.join(file_name).with_extension(ext);
        let mut target = File::options().create(true).write(true).open(path)?;
        target.write_all(&data)?;
        Ok(())
    }

    fn get_data<R, P>(&self, mut dump: R, provider: &P) -> Result<Vec<u8>>
    where
        R: Read,
        P: DataProvider,
    {
        let mut data = Vec::new();
        let mut buffer = [0; 1024];
        let progress = self.create_progress(provider)?;
        while let Ok(size) = dump.read(&mut buffer) {
            if size == 0 {
                break;
            }
            data.write_all(&buffer[..size])?;
            self.total.inc(size as u64);
            if let Some(p) = &progress {
                p.inc(size as u64);
            }
        }
        if let Some(p) = &progress {
            p.finish();
        }
        Ok(data)
    }

    fn start(&self) -> Result<()> {
        let mut tasks = Vec::new();
        let (tx, rx) = crossbeam_channel::unbounded();

        {
            let state = self.clone();
            let task = thread::spawn(move || {
                for matcher in &state.command.matchers {
                    for entry in glob(matcher)? {
                        let path = entry.map_err(|_| Error::Path)?;
                        if !path.is_file() {
                            continue;
                        }
                        let p = FileProvider::new(path).map_err(|_| Error::Path)?;
                        let len = state.total.length().unwrap_or(0);
                        state.total.set_length(len + p.get_size());
                        tx.send(p)?;
                    }
                }
                anyhow::Ok(())
            });
            tasks.push(task);
        }

        for _ in 1..=self.command.worker {
            let rx = rx.clone();
            let state = self.clone();
            let task = thread::spawn(move || {
                while let Ok(w) = rx.recv() {
                    state.dump(&w)?;
                }
                anyhow::Ok(())
            });
            tasks.push(task);
        }
        for task in tasks {
            task.join().unwrap()?;
        }
        self.finish();
        Ok(())
    }
}

fn main() -> Result<()> {
    let command = Command::parse();
    command.invalid()?;

    let program = Program::new(command)?;
    program.start()
}
