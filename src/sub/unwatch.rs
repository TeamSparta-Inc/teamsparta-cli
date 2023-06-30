use crate::{
    cli::{UnwatchCommand, WatchService},
    config::WatchServiceConfig,
};
use std::{fs, process::Stdio};

pub fn run_unwatch_command(unwatch_opts: UnwatchCommand, watch_service_config: WatchServiceConfig) {
    let WatchServiceConfig { pipeline, .. } = watch_service_config;
    let pipeline_name = match unwatch_opts.service {
        WatchService::Online => pipeline.online,
        WatchService::OnlineTest => pipeline.online_test,
        WatchService::Swc => pipeline.swc,
        WatchService::SwcTest => pipeline.swc_test,
        WatchService::Hhv2 => pipeline.hhv2,
        WatchService::Hhv2Test => pipeline.hhv2_test,
        WatchService::Nbc => pipeline.nbc,
        WatchService::NbcTest => pipeline.nbc_test,
        WatchService::Intellipick => pipeline.intellipick,
        WatchService::IntellipickTest => pipeline.intellipick_test,
        WatchService::H99 => pipeline.h99,
        WatchService::H99Test => pipeline.h99_test,
    };
    let pid_path = format!("/tmp/watch/{}/daemon.pid", pipeline_name);
    let watch_daemon_pid = fs::read_to_string(&pid_path).unwrap_or_else(|_| {
        eprintln!(
            "종료할 데몬프로세스 아이디가 존재하지 않습니다.\n종료를 시도한 관측 대상: {}",
            pipeline_name
        );
        std::process::exit(1);
    });
    println!(
        "{}을(를) 관측 중인 데몬프로세스를 종료합니다: pid {}",
        pipeline_name, watch_daemon_pid
    );
    let kill = std::process::Command::new("kill")
        .arg("-9")
        .arg(watch_daemon_pid.trim())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("데몬프로세스 제거에 실패했습니다. 권한 상향(sudo)를 고려해보세요");

    fs::remove_file(pid_path).expect("pid 파일 제거에 실패했습니다");
    println!("{}", String::from_utf8_lossy(&kill.stdout));
    eprintln!("{}", String::from_utf8_lossy(&kill.stderr));
}
