use clap::{crate_authors, Clap};

static VERSION: &str = env!("VERSION_WITH_GIT_HASH");
static AUTHORS: &str = crate_authors!();
static DESCRIPTION: &str = crate_description!();

#[derive(Clap)]
#[clap(author = AUTHORS, version = VERSION, about = DESCRIPTION)]
pub(crate) struct Args {
    /// Show verbose output at a level or higher. -v:  DEBUG, -vv: TRACE
    #[clap(long, short, parse(from_occurrences))]
    pub(crate) verbose: u8,
    /// Supress output at a level or lower. -q: INFO, -qq: WARN, -qqq: ERROR (i.e. everything)
    #[clap(long, short, overrides_with = "verbose", parse(from_occurrences))]
    pub(crate) quiet: u8,
}
