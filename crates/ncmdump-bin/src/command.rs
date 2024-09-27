use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::errors::Error;

#[derive(Clone, Debug, Default, Parser)]
#[command(name = "ncmdump", bin_name = "ncmdump", about, version)]
pub(crate) struct Command {
    /// Specified the files to convert.
    #[arg(value_name = "FILES")]
    pub(crate) matchers: Vec<String>,

    /// Specified the output directory.
    /// Default it's the same directory with input file.
    #[arg(short = 'o', long = "output")]
    pub(crate) output: Option<String>,

    /// Verbosely list files processing.
    #[arg(short = 'v', long = "verbose")]
    pub(crate) verbose: bool,

    /// The process work count.
    /// It should more than 0 and less than 9.
    #[arg(short = 'w', long = "worker", default_value = "1")]
    pub(crate) worker: usize,
}

impl Command {
    pub(crate) fn invalid(&self) -> Result<(), Error> {
        // Check argument worker
        if self.worker < 1 || self.worker > 8 {
            return Err(Error::Worker);
        }

        // Check argument matchers
        if self.matchers.is_empty() {
            return Err(Error::NoFile);
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn items(&self) -> Result<Vec<PathBuf>, Error> {
        let mut paths = Vec::new();
        for matcher in &self.matchers {
            for entry in glob::glob(matcher)? {
                let path = entry?;
                if !path.is_file() {
                    continue;
                }
                paths.push(path)
            }
        }
        Ok(paths)
    }

    #[cfg(not(target_os = "windows"))]
    pub(crate) fn items(&self) -> Result<Vec<PathBuf>, Error> {
        let mut paths = Vec::new();
        for matcher in &self.matchers {
            let path = PathBuf::from(matcher);
            if !path.is_file() {
                continue;
            }
            paths.push(path)
        }
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::errors::Error;
    use crate::Command;

    #[test]
    fn test_empty_input_files_err() -> Result<()> {
        let command = Command {
            matchers: vec![],
            worker: 1,
            ..Default::default()
        };
        let result = command.invalid();
        assert!(result.is_err_and(|err| err == Error::NoFile));
        Ok(())
    }

    #[test]
    fn test_invalid_worker_ok() -> Result<()> {
        let works = [1, 2, 3, 4, 5, 6, 7, 8];
        for worker in works {
            let command = Command {
                matchers: vec![String::new()],
                worker,
                ..Default::default()
            };
            let result = command.invalid();
            assert!(result.is_ok());
        }
        Ok(())
    }

    #[test]
    fn test_invalid_worker_err() -> Result<()> {
        let works = [0, 9, 10, 15, 100, 199];
        for worker in works {
            let command = Command {
                matchers: vec![String::new()],
                worker,
                ..Default::default()
            };
            let result = command.invalid();
            assert!(result.is_err_and(|err| err == Error::Worker));
        }
        Ok(())
    }
}
