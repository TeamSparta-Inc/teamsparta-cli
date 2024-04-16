use clap::Parser;
use cli::{Cli, Subcommand};
use config::Config;

mod cli;
mod common;
mod config;
mod sub;

#[tokio::main]
async fn main() {
    let opts = Cli::parse();
    let config = Config::new().expect("failed to get config");

    match opts.subcommand {
        Subcommand::Dump(dump_opts) => sub::dump::run_dump(dump_opts, config.mongo_dump),
        Subcommand::Resize(resize_opts) => sub::resize::run_resize(resize_opts),
        Subcommand::Compress(compress_opts) => sub::compress::run_compress(compress_opts),
        Subcommand::Webpify(webpify_opts) => sub::webpify::run_webpify(webpify_opts),
        Subcommand::Cred(cred_opts) => sub::credential::run_credential(cred_opts).await,
    }
}
