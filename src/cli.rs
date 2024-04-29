use clap::{Parser, ValueEnum};
use std::path::PathBuf;

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
        about = "source DB로부터 target DB로 mongodump & restore를 실행합니다"
    )]
    Dump(DumpCommand),
    #[command(
        name = "resize",
        about = "이미지 파일 해상도 변경\nsprt resize -i path/to/input_dir [-f file_name] -o path/to/output_dir -w 1920 -h 1080"
    )]
    Resize(ResizeCommand),
    #[command(
        name = "compress",
        about = "이미지 손실/무손실 압축(손실 압축시 tiny png 사이트 방식)\nsprt compress -i path/to/input_dir [-f file_name] -o path/to/output_dir -d"
    )]
    Compress(CompressCommand),
    #[command(
        name = "webpify",
        about = "png/jpeg를 webp로 변환\nsprt webpify -i path/to/input_dir -o path/to/output_dir"
    )]
    Webpify(WebpifyCommand),
    #[command(
        name = "cred",
        about = "개발용 credential 반환\n등록: sprt cred -m register -u [USER_NAME] -p [PASSWORD] -c [CONFIRM_PASSWORD] --aws-access-key-id [ACCESS_KEY_ID] --aws-secret-access-key [SECRET_ACCESS_KEY]"
    )]
    Cred(CredCommand),
}

#[derive(Parser)]
pub struct DumpCommand {
    #[arg(short, long)]
    pub service: String,
    #[arg(short, long, num_args(0..))]
    pub collections: Option<Vec<String>>,
    #[arg(short, long)]
    pub family: Option<String>,
}

#[derive(Parser)]
pub struct ResizeCommand {
    #[arg(short, long)]
    pub input_dir: PathBuf,
    #[arg(short, long)]
    pub output_dir: PathBuf,
    #[arg(short, long)]
    pub width: u32,
    #[arg(short, long)]
    pub height: u32,
    #[arg(short, long)]
    pub file_name: Option<String>,
}

#[derive(Parser)]
pub struct CompressCommand {
    #[arg(short, long)]
    pub input_dir: PathBuf,
    #[arg(short, long)]
    pub output_dir: PathBuf,
    #[arg(short, long)]
    pub file_name: Option<String>,
    #[arg(short, long, default_value_t = 12, value_parser = 1..=12)]
    pub level: i64,
    #[arg(short, long)]
    pub drop_color: bool,
    #[arg(short, long, default_value_t = 4, value_parser = 1..=10)]
    pub speed: i64,
    #[arg(short, long, default_value_t = 65, value_parser = 1..=100)]
    pub quality: i64,
}
#[derive(Parser)]
pub struct WebpifyCommand {
    #[arg(short, long, num_args(1..))]
    pub input_dir: PathBuf,
    #[arg(short, long)]
    pub output_dir: PathBuf,
}

#[derive(Parser)]
pub struct CredCommand {
    #[arg(short, long, value_enum)]
    pub mode: CredMode,
    #[arg(short, long)]
    pub user_name: Option<String>,
    #[arg(short, long)]
    pub password: Option<String>,
    #[arg(short, long)]
    pub confirm_password: Option<String>,
    #[arg(long)]
    pub aws_access_key_id: Option<String>,
    #[arg(long)]
    pub aws_secret_access_key: Option<String>,
    #[arg(long)]
    pub arn: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(short, long)]
    pub region: Option<String>,
}

#[derive(ValueEnum, Clone)]
pub enum CredMode {
    Develop,
    Private,
    Register,
    Update,
    Awscli,
    Session,
    Revoke,
    AddProfile,
    UpdateProfile,
}
