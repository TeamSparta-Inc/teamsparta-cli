use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone)]
pub enum DumpServices {
    Chang,
    Online,
    Swc,
    Hhv2,
    Nbc,
    Scc,
    Intellipick,
    Backoffice,
    BackofficeBootcamp,
}
#[derive(ValueEnum, Clone)]
pub enum WatchService {
    Online,
    OnlineTest,
    Swc,
    SwcTest,
    Hhv2,
    Hhv2Test,
    Nbc,
    NbcTest,
    Intellipick,
    IntellipickTest,
    H99,
    H99Test,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Parser)]
pub enum Subcommand {
    #[command(
        name = "dump",
        about = "production DB로부터 local로 mongodump를 실행합니다"
    )]
    Dump(DumpCommand),
    #[command(
        name = "watch",
        about = "AWS Codepipeline 배포가 완료되면 슬랙 채널을 통해 통지받습니다"
    )]
    Watch(WatchCommand),
    #[command(name = "unwatch", about = "AWS Codepipeline 배포 감시를 취소합니다")]
    Unwatch(UnwatchCommand),
    #[command(name = "resize", about = "이미지 파일 사이즈 변경")]
    Resize(ResizeCommand),
}

#[derive(Parser)]
pub struct DumpCommand {
    #[arg(short, long)]
    pub service: DumpServices,
    #[arg(short, long)]
    pub port: Option<u32>,
    #[arg(short, long, num_args(0..))]
    pub collection: Option<Vec<String>>,
}

#[derive(Parser)]
pub struct WatchCommand {
    #[arg(short, long)]
    pub service: WatchService,
    #[arg(short, long, num_args(0..))]
    pub notify: Option<Vec<String>>,
}

#[derive(Parser)]
pub struct UnwatchCommand {
    #[arg(short, long)]
    pub service: WatchService,
}

#[derive(Parser, Debug)]
pub struct ResizeCommand {
    #[arg(short, long)]
    pub input: PathBuf,
    #[arg(short, long)]
    pub output: PathBuf,
    #[arg(short, long)]
    pub dir: Option<bool>,
    #[arg(short, long)]
    pub width: usize,
    #[arg(short, long)]
    pub height: usize,
}
