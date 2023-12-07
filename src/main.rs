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
        Subcommand::Resize(resize_opts) => sub::resize::run_resize(resize_opts),
        Subcommand::Compress(compress_opts) => sub::compress::run_compress(compress_opts),
        Subcommand::Unused(unused_opts) => sub::unused::run_unused(unused_opts),
        Subcommand::Webpify(webpify_opts) => sub::webpify::run_webpify(webpify_opts),
    }
}
