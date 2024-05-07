use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use clap::Parser;

use ncmdump::{Ncmdump, QmcDump};
use ncmdump::utils::FileType;

use crate::command::Command;
use crate::errors::Error;
use crate::metadata::{FlacMetadata, Metadata, Mp3Metadata};
use crate::provider::{DataProvider, FileProvider};
use crate::state::State;

mod command;
mod errors;
mod metadata;
mod provider;
mod state;
mod utils;

/// The global program
#[derive(Clone)]
struct Program {
    command: Arc<Command>,
    state: Arc<State>,
}

impl Program {
    /// Create new command progress.
    fn new(command: Command) -> Result<Self> {
        let state = State::try_from(&command)?;
        Ok(Self {
            command: Arc::new(command),
            state: Arc::new(state),
        })
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
        let mut target = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        if provider.get_format() == FileType::Ncm {
            let file = File::open(provider.get_path())?;
            let mut dump = Ncmdump::from_reader(file)?;
            let image = dump.get_image()?;
            let info = dump.get_info()?;
            if ext == "mp3" {
                let buffer = Mp3Metadata::new(&info, &image, &data).inject_metadata(data)?;
                target.write_all(&buffer)?;
            } else if ext == "flac" {
                let buffer = FlacMetadata::new(&info, &image, &data).inject_metadata(data)?;
                target.write_all(&buffer)?;
            }
        }
        Ok(())
    }

    fn get_data<R, P>(&self, mut dump: R, provider: &P) -> Result<Vec<u8>>
    where
        R: Read,
        P: DataProvider,
    {
        let mut data = Vec::new();
        let mut buffer = [0; 1024];
        let progress = self.state.create_progress(provider)?;
        while let Ok(size) = dump.read(&mut buffer) {
            if size == 0 {
                break;
            }
            data.write_all(&buffer[..size])?;
            self.state.inc(size as u64);
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

        let items = self.command.items()?;
        let state = self.state.clone();
        tasks.push(thread::spawn(move || {
            for path in items {
                let provider = FileProvider::new(path).map_err(|_| Error::Path)?;
                state.inc_length(provider.get_size());
                tx.send(provider)?;
            }
            anyhow::Ok(())
        }));

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
        Ok(())
    }
}

fn main() -> Result<()> {
    let command = Command::parse();
    command.invalid()?;

    let program = Program::new(command)?;
    program.start()
}
