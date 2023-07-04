use crate::{
    cli::{WatchCommand, WatchService},
    config::WatchServiceConfig,
};
use aws_sdk_codepipeline as codepipeline;
use chrono::{DateTime, Utc};
use codepipeline::{primitives, Client};
use daemonize::Daemonize;
use serde::Deserialize;
use std::{str::from_utf8, fs::{File, self}, time::Duration, time::SystemTime, process::Command, path};
use tokio::time::sleep;
use colored::*;

#[derive(Deserialize, Debug)]
struct SourceSummary {
    #[serde(rename = "ProviderType", default)]
    _provider_type: String,
    #[serde(rename = "CommitMessage", default)]
    commit_message: String,
}

macro_rules! err_msg {
    ($em:literal) => {
        format_args!("{} AWS 응답의 일시적인 장애일 수 있으니 잠시 후 재시도 해보세요", $em)
    };
}

macro_rules! daemon_root {
    ($path:literal) => {
        format!("/tmp/watch/{}", $path)
    };
    ($path:expr) => {
        format!("/tmp/watch/{}", $path)
    };
}

async fn get_last_execution_data(
    codepipeline_client: &Client,
    pipeline_name: &str,
) -> Option<(String, primitives::DateTime, primitives::DateTime, String)> {
    let pipeline = codepipeline_client
        .list_pipeline_executions()
        .pipeline_name(pipeline_name)
        .send()
        .await;

    let pipeline_state_output = pipeline.unwrap_or_else(|e| {
        panic!("pipeline state output을 unwrap할 수 없습니다:{}", e);
    });

    let latest_summary = pipeline_state_output
        .pipeline_execution_summaries()
        .unwrap_or_else(||panic!("{}", err_msg!(
            "파이프라인 실행 요약을 가져오는데에 실패했습니다."
        )))
        .get(0)
        .unwrap_or_else(|| panic!("{}", err_msg!(
            "가장 최근의 pipeline 실행 요약을 가져오는데에 실패했습니다."
        )));

    let revision_summary = latest_summary
        .source_revisions()
        .unwrap_or_else(||panic!("{}",err_msg!(
            "pipeline 실행 요약의 source revision을 가져오는데에 실패했습니다."
        )))
        .get(0)
        .unwrap_or_else(|| panic!("{}",err_msg!(
            "source revision의 최근 내역을 가져오는데에 실패했습니다."
        )))
        .revision_summary()
        .unwrap_or_else(|| panic!("{}",err_msg!(
            "최근의 source revision 요약을 가져오는데에 실패했습니다."
        )));

    let pipeline_status = latest_summary
        .status()
        .unwrap_or_else(|| panic!("{}",err_msg!(
            "파이프라인 실행 상태를 가져오는데에 실패했습니다."
        )))
        .to_owned()
        .as_str()
        .to_string();
    let start_time = latest_summary.start_time().unwrap().to_owned();
    let last_update_time = latest_summary.last_update_time().unwrap().to_owned();
    let SourceSummary { commit_message, .. }: SourceSummary =
        serde_json::from_str(revision_summary).unwrap();

    Some((
        pipeline_status,
        start_time,
        last_update_time,
        commit_message,
    ))
}

impl Default for SourceSummary {
    fn default() -> Self {
        SourceSummary {
            _provider_type: "unknown".to_string(),
            commit_message: "unknown".to_string(),
        }
    }
}

pub fn run_watch(watch_opts: WatchCommand, watch_service_config: WatchServiceConfig) {
    let WatchServiceConfig {pipeline, slack} = watch_service_config;
    let pipeline_name = match watch_opts.service {
        WatchService::Online => pipeline.online,
        WatchService::OnlineTest =>pipeline.online_test,
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
    let parent_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio parent runtime 빌드에 실패했습니다");

    let (
        stdout_path, 
        stderr_path, 
        daemon_pid_path
    ) = (
            daemon_root!("daemon.stdout.log"), 
            daemon_root!("daemon.stderr.log"), 
            format!("{}/{}", daemon_root!(pipeline_name), "daemon.pid")
        );

    parent_runtime.block_on(async {
        if let Ok(exists) = path::Path::new(&daemon_pid_path).try_exists() {
            if exists {
                println!("이미 해당 작업이 실행중입니다. 만약 문제가 있다면 unwatch 명령어를 실행하신 후에 재시도해주세요");
                std::process::exit(0);
            }
        }

        let aws_config = ::aws_config::load_from_env().await;
        let codepipeline_client = codepipeline::Client::new(&aws_config);
        let (status, ..) = get_last_execution_data(&codepipeline_client, &pipeline_name)
            .await
            .unwrap_or_else(|| panic!("{}",err_msg!("배포 상태 점검에 실패했습니다.")));
        if status != "InProgress" {
            let not_proceed_word = "배포가 진행 중이지 않기 때문에 배포를 관측할 수 없습니다. 명령을 종료합니다.".bright_blue().bold();
            println!("{}", not_proceed_word);
            std::process::exit(0);
        } else {
            let init_word = "배포 관측을 시작합니다. 배포가 종료되면 채널 '#배포_알림' 에서 알려드리겠습니다.\n\n백그라운드 실행 중인 로그 정보는 다음의 경로에 위치합니다.\n".bright_blue().bold();
            let log_path_string = format!("- {}\n- {}", stdout_path, stderr_path).bold();
            println!("{}{}", init_word, log_path_string);
        }
    });

    fs::create_dir_all(daemon_root!(pipeline_name)).unwrap();
    File::create(&daemon_pid_path).unwrap();

    let stdout = File::create(&stdout_path).unwrap();
    let stderr = File::create(&stderr_path).unwrap();
    let daemon = 
        Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .pid_file(&daemon_pid_path);

    daemon
    .start()
    .unwrap_or_else(|e| panic!("데몬프로세스 실행에 실패했습니다:\n{}", e));

    let child_runtime = 
        tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio child runtime 빌드에 실패했습니다");

    child_runtime.block_on(async {
        let aws_config = ::aws_config::load_from_env().await;
        let codepipeline_client = codepipeline::Client::new(&aws_config);

        let start_utc = Utc::now().timestamp_millis();
        
        let (status, start_time, last_update_time, commit_message) = loop {
            sleep(Duration::from_secs(15)).await;

            let mut retried = 0;
            let pipeline_info = loop {
                let last_execution_data =
                    get_last_execution_data(&codepipeline_client, &pipeline_name).await;

                if let Some(v) = last_execution_data {
                    break v;
                }
                if retried > 5 {
                    panic!("AWS Codepipeline 정보를 가져오는데에 실패했습니다.")
                }
                retried += 1;
            };

            if pipeline_info.0 != "InProgress" || start_utc - Utc::now().timestamp_millis() > 1000 * 60 * 30 {
                break pipeline_info;
            }
        };

        let start_time_str: DateTime<Utc> = SystemTime::try_from(start_time).unwrap().into();
        let last_update_time_str: DateTime<Utc> = SystemTime::try_from(last_update_time).unwrap().into();
        let webhook_prefix = if status == "InProgress" {
            "배포가 시작된지 30분이 경과했지만 종료되지 않았습니다. 직접 상태를 확인해주세요. 배포 관측 작업을 종료합니다."
        } else {
            "배포가 종료되었습니다."
        };

        let notify_names: Vec<String> = watch_opts.notify.unwrap_or(vec![]);
        let notify_ids = notify_names
            .iter()
            .map(|name| {
                    slack
                    .known_users
                    .get(name)
                    .unwrap_or(name)
            })
            .fold(
                format!(" <@{}>",slack.user_id),
                |user_ids, user_id_or_name| format!("{}<@{}>", user_ids, user_id_or_name),
            );

        let body = format!("{{\"text\":\"{}\"}}", format_args!(
            "{}\n{}: {}\n🛫: {}\n🛬: {}\n📋: {}\n🎯:{}",
            webhook_prefix,
            pipeline_name,
            match &status[..] {
                "Succeeded" => "🟢",
                "InProgress" => "🟠",
                _ => "🔴"
            },
            start_time_str.format("%Y-%m-%d %H:%M:%S"),
            last_update_time_str.format("%Y-%m-%d %H:%M:%S"),
            commit_message,
            notify_ids
        ));

        let curl_output = 
            Command::new("curl")
            .args(["-d", &body])
            .args(["-H", "Content-Type: application/json"])
            .args(["-X", "POST"])
            .arg(slack.webhook_url)
            .output()
            .unwrap_or_else(|e| panic!("curl failed: {}", e));

        println!("{}", from_utf8(&curl_output.stdout).unwrap());
        eprintln!("{}", from_utf8(&curl_output.stderr).unwrap());

        fs::remove_file(daemon_pid_path).unwrap_or_else(|e|panic!("pid 파일 제거에 실패했습니다:{}", e));
    });
}


