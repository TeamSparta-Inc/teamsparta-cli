use crate::{
    cli::{CredCommand, CredMode},
    exit_with_error,
};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;
const MILLISECONDS_IN_AN_HOUR: u128 = 3600 * 1000;
const GCS_CREDENTIAL_URL: &str = "https://gcs.spartacodingclub.com/credential/";
const SESSION_SUFFIX: &str = "sprt/session";

fn path(path: &str) -> String {
    let base = Path::new(GCS_CREDENTIAL_URL);
    base.join(path).to_str().expect("invalid url").to_owned()
}

pub async fn run_credential(cred_opts: CredCommand) {
    let client = reqwest::Client::new();
    let mut body = HashMap::new();
    let home_dir = dirs::home_dir().expect("failed to get home dir");
    let session_path = home_dir.join(SESSION_SUFFIX);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(session_path);

    let mut session_key = String::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("We are in the past");
    match file {
        Ok(mut file_content) => {
            let mut content = String::new();
            file_content.read_to_string(&mut content).unwrap();

            if content.is_empty() {
                let new_session_key = Uuid::new_v4().to_string();
                let utc_millis = now.as_millis().to_string();

                writeln!(file_content, "{} {}", new_session_key, utc_millis).unwrap();
                session_key = new_session_key;
            } else {
                let session_id_and_updated_at: Vec<&str> = content
                    .lines()
                    .last()
                    .expect("cannot take last line of session")
                    .split(' ')
                    .collect();

                if session_id_and_updated_at.len() < 2 {
                    exit_with_error!("session exists but malformed")
                }

                let (existing_session_key, updated_at) = (
                    session_id_and_updated_at.first().unwrap().to_owned(),
                    session_id_and_updated_at.get(1).unwrap().to_owned(),
                );

                if let Ok(updated_at_millis) = updated_at.parse::<u128>() {
                    let now_millis = now.as_millis();
                    if now_millis < updated_at_millis {
                        exit_with_error!("session exists but maybe system time exploited")
                    }
                    if now_millis - updated_at_millis > MILLISECONDS_IN_AN_HOUR {
                        let new_session_key = Uuid::new_v4().to_string();
                        let utc_millis = now.as_millis().to_string();

                        writeln!(file_content, "{} {}", new_session_key, utc_millis).unwrap();
                        session_key = new_session_key;
                    } else {
                        session_key = existing_session_key.into()
                    }
                }
            }
        }
        Err(open_err) => {
            eprintln!("cannot use credential session by error: {}\nif \"Permission denied\", try sudo mode", open_err);
        }
    }

    match cred_opts.mode {
        CredMode::Register => {
            if cred_opts.user_name.is_none()
                || cred_opts.password.is_none()
                || cred_opts.confirm_password.is_none()
                || cred_opts.aws_access_key_id.is_none()
                || cred_opts.aws_secret_access_key.is_none()
            {
                exit_with_error!("Values missing! register needs --user-name --password --aws_access_key_id --aws_secret_access_key")
            }

            body.insert("user_name", cred_opts.user_name.unwrap());
            body.insert("password", cred_opts.password.unwrap());
            body.insert("confirm_password", cred_opts.confirm_password.unwrap());
            body.insert("aws_access_key_id", cred_opts.aws_access_key_id.unwrap());
            body.insert(
                "aws_secret_access_key",
                cred_opts.aws_secret_access_key.unwrap(),
            );

            let result = client
                .post(path("register"))
                .json(&body)
                .send()
                .await
                .expect("failed to register");

            if !result.status().is_success() {
                exit_with_error!("register failed")
            }
            println!("register succeeded")
        }
        CredMode::Develop => {
            let user_name = cred_opts.user_name.unwrap_or_default();
            let password = cred_opts.password.unwrap_or_default();

            body.insert("user_name", user_name);
            body.insert("password", password);
            body.insert("private_key", session_key);

            let result = client
                .post(path("local-dev"))
                .json(&body)
                .send()
                .await
                .expect("failed to fetch local development credentials");

            if !result.status().is_success() {
                exit_with_error!("failed to fetch local development credentials")
            }

            let response_body = result.text().await.expect("failed to parse response");
            println!("{}", response_body)
        }
        CredMode::Private => {
            let user_name = cred_opts.user_name.unwrap_or_default();
            let password = cred_opts.password.unwrap_or_default();

            body.insert("user_name", user_name);
            body.insert("password", password);
            body.insert("private_key", session_key);

            let result = client
                .post(path("private"))
                .json(&body)
                .send()
                .await
                .expect("failed to fetch private credentials");

            let status = result.status();
            let response_text = result.text().await.expect("failed to text response");

            if !status.is_success() {
                println!("{}", response_text);
                exit_with_error!("failed to fetch private credentials")
            }

            println!("{}", response_text)
        }
        CredMode::Update => {
            if cred_opts.user_name.is_none()
                || cred_opts.password.is_none()
                || cred_opts.aws_access_key_id.is_none()
                || cred_opts.aws_secret_access_key.is_none()
            {
                exit_with_error!("credential update need --user-name --password --aws-access-key-id --aws-secret_access_key")
            }

            body.insert("user_name", cred_opts.user_name.unwrap());
            body.insert("password", cred_opts.password.unwrap());
            body.insert("aws_access_key_id", cred_opts.aws_access_key_id.unwrap());
            body.insert(
                "aws_secret_access_key",
                cred_opts.aws_secret_access_key.unwrap(),
            );

            let result = client
                .post(path("update"))
                .json(&body)
                .send()
                .await
                .expect("failed to update credentials");

            let status = result.status();
            let response_text = result.text().await.expect("failed to text response");

            if !status.is_success() {
                println!("{}", response_text);
                exit_with_error!("failed to update credentials")
            }

            println!("{}", response_text)
        }
        CredMode::Awscli => {
            let user_name = cred_opts.user_name.unwrap_or_default();
            let password = cred_opts.password.unwrap_or_default();

            body.insert("user_name", user_name);
            body.insert("password", password);
            body.insert("private_key", session_key);

            let result = client
                .post(path("aws-cli"))
                .json(&body)
                .send()
                .await
                .expect("failed to fetch private credentials");

            let status = result.status();
            let response_text = result.text().await.expect("failed to text response");

            if !status.is_success() {
                println!("{}", response_text);
                exit_with_error!("failed to fetch private credentials")
            }

            println!("{}", response_text)
        }
        CredMode::Session => {
            if session_key.is_empty() {
                exit_with_error!("cannot read session key. try sudo mode")
            }
            if cred_opts.user_name.is_none() || cred_opts.password.is_none() {
                exit_with_error!("session mode needs --user-name --password")
            }

            let user_name = cred_opts.user_name.unwrap_or_default();
            let password = cred_opts.password.unwrap_or_default();

            body.insert("user_name", user_name);
            body.insert("password", password);
            body.insert("private_key", session_key);

            let result = client
                .post(path("private"))
                .json(&body)
                .send()
                .await
                .expect("failed to fetch private credentials");

            let status = result.status();
            let response_text = result.text().await.expect("failed to text response");

            if !status.is_success() {
                println!("{}", response_text);
                exit_with_error!("failed to fetch private credentials")
            }

            println!("1hour session made")
        }
    }
}
