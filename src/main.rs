use clap::Parser;
use cli::{Cli, Subcommand};
use config::Config;

mod cli;
mod config;
mod sub;

fn main() {
    let opts = Cli::parse();
    let config = Config::new().expect("failed to get config");

    match opts.subcommand {
        Subcommand::Dump(dump_opts) => sub::dump::run_dump(dump_opts, config.mongo_dump),
        Subcommand::Watch(watch_opts) => sub::watch::run_watch(watch_opts, config.watch),
        Subcommand::Unwatch(unwatch_opts) => sub::unwatch::run_unwatch(unwatch_opts, config.watch),
        Subcommand::Resize(resize_opts) => sub::resize::run_resize(resize_opts),
    }
}
