use crate::{
    cli::{CredCommand, CredMode},
    exit_with_error,
};
use std::{
    collections::HashMap,
    env,
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;
const MILLISECONDS_IN_A_DAY: u128 = 24 * 3600 * 1000;
const GCS_CREDENTIAL_URL: &str = "https://gcs.spartacodingclub.com/credential/";
const SESSION_SUFFIX: &str = "sprt/session";

fn path(path: &str) -> String {
    let credential_uri: &str = if env::var("ENV").unwrap_or_default() == "dev" {
        "http://localhost:8080/credential/"
    } else {
        GCS_CREDENTIAL_URL
    };
    let base = Path::new(credential_uri);
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
                    if now_millis - updated_at_millis > MILLISECONDS_IN_A_DAY {
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
        Err(_) => {
            eprintln!("couldn't use session. if you want to use session, try sudo mode");
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
                exit_with_error!("Value missing! register needs --user-name --password --confirm-password --aws-access-key-id --aws-secret-access-key")
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
                exit_with_error!("credential update need --user-name --password --aws-access-key-id --aws-secret-access-key")
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
            let profile = cred_opts.profile.unwrap_or_default();

            body.insert("user_name", user_name);
            body.insert("password", password);
            body.insert("private_key", session_key);
            body.insert("profile_name", profile);

            let result = client
                .post(path("aws-cli"))
                .json(&body)
                .send()
                .await
                .expect("failed to fetch aws-cli credentials");

            let status = result.status();
            let response_text = result.text().await.expect("failed to text response");

            if !status.is_success() {
                println!("{}", response_text);
                exit_with_error!("failed to fetch aws-cli credentials")
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
            let profile = cred_opts.profile.unwrap_or_default();

            body.insert("user_name", user_name);
            body.insert("password", password);
            body.insert("private_key", session_key);
            body.insert("profile_name", profile);

            let result = client
                .post(path("session"))
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

            println!("24 hours session made")
        }
        CredMode::Revoke => {
            if session_key.is_empty() {
                exit_with_error!("cannot read session key. try sudo mode")
            }

            body.insert("private_key", session_key);

            let result = client
                .post(path("revoke"))
                .json(&body)
                .send()
                .await
                .expect("failed to revoke session");

            let status = result.status();
            let response_text = result.text().await.expect("failed to text response");

            if !status.is_success() {
                println!("{}", response_text);
                exit_with_error!("failed to revoke session")
            }

            println!("{}", response_text)
        }
        CredMode::AddProfile => {
            if cred_opts.user_name.is_none()
                || cred_opts.password.is_none()
                || cred_opts.role_arn.is_none()
                || cred_opts.profile.is_none()
            {
                exit_with_error!(
                    "add-profile needs --user-name --password --role-arn --profile --region(optional)"
                )
            }

            body.insert("user_name", cred_opts.user_name.unwrap());
            body.insert("password", cred_opts.password.unwrap());
            body.insert("role_arn", cred_opts.role_arn.unwrap());
            body.insert("profile_name", cred_opts.profile.unwrap());
            body.insert("region", cred_opts.region.unwrap_or_default());

            let response = client
                .post(path("add-profile"))
                .json(&body)
                .send()
                .await
                .expect("failed to add profile");

            let status = response.status();
            let response_text = response.text().await.expect("failed to text response");

            if !status.is_success() {
                println!("{}", response_text);
                exit_with_error!("failed to add profile")
            }

            println!("{}", response_text)
        }
        CredMode::UpdateProfile => {
            if cred_opts.user_name.is_none()
                || cred_opts.password.is_none()
                || cred_opts.role_arn.is_none()
                || cred_opts.profile.is_none()
            {
                exit_with_error!("update-profile needs --user-name --password --role-arn --profile --region(optional)")
            }

            body.insert("user_name", cred_opts.user_name.unwrap());
            body.insert("password", cred_opts.password.unwrap());
            body.insert("role_arn", cred_opts.role_arn.unwrap());
            body.insert("profile_name", cred_opts.profile.unwrap());
            body.insert("region", cred_opts.region.unwrap_or_default());

            let response = client
                .post(path("update-profile"))
                .json(&body)
                .send()
                .await
                .expect("failed to update profile");

            let status = response.status();
            let response_text = response.text().await.expect("failed to text response");

            if !status.is_success() {
                println!("{}", response_text);
                exit_with_error!("failed to update profile")
            }

            println!("{}", response_text)
        }
    }
}
