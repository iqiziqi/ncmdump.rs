use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::command::Command;
use crate::errors::Error;
use crate::provider::DataProvider;

const TOTAL_PSTYPE: &str = "[{bar:40.cyan}] |{percent:>3!}%| {bytes:>10!}/{total_bytes:10!}";
const SINGLE_PSTYPE: &str = "[{bar:40.cyan}] |{percent:>3!}%| {bytes:>10!}/{total_bytes:10!} {msg}";

#[derive(Clone)]
pub(crate) struct State {
    verbose: bool,
    group: MultiProgress,
    total: ProgressBar,
}

impl State {
    /// Create a new progress.
    pub(crate) fn create_progress<P>(&self, provider: &P) -> Result<Option<ProgressBar>>
    where
        P: DataProvider,
    {
        if !self.verbose {
            return Ok(None);
        }
        let style = ProgressStyle::with_template(SINGLE_PSTYPE)?;
        let progress = self
            .group
            .insert_from_back(1, ProgressBar::new(provider.get_size()).with_style(style));
        progress.set_message(provider.get_name());
        Ok(Some(progress))
    }

    pub(crate) fn inc_length(&self, num: u64) {
        self.total.inc_length(num);
    }

    pub(crate) fn inc(&self, num: u64) {
        self.total.inc(num);
    }
}

impl TryFrom<&Command> for State {
    type Error = Error;

    fn try_from(command: &Command) -> std::result::Result<Self, Self::Error> {
        let group = MultiProgress::new();
        let style = ProgressStyle::with_template(TOTAL_PSTYPE).unwrap();
        let total = group.add(ProgressBar::new(0).with_style(style));
        Ok(Self {
            verbose: command.verbose,
            group,
            total,
        })
    }
}
