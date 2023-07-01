use crate::{
    cli::{DumpCommand, DumpServices},
    config::MongoDumpServiceConfig,
};
use colored::*;
use std::process::{exit, Command, Stdio};

pub fn run_dump(dump_opts: DumpCommand, dump_service_config: MongoDumpServiceConfig) {
    let dump_instruction = match dump_opts.service {
        DumpServices::Chang => dump_service_config.chang,
        DumpServices::Hhv2 => dump_service_config.hhv2,
        DumpServices::Nbc => dump_service_config.nbc,
        DumpServices::Online => dump_service_config.online,
        DumpServices::Scc => dump_service_config.scc,
        DumpServices::Swc => dump_service_config.swc,
        DumpServices::Intellipick => dump_service_config.intellipick,
        DumpServices::Backoffice => dump_service_config.backoffice,
        DumpServices::BackofficeBootcamp => dump_service_config.backoffice_bootcamp,
    };

    let mut dump_command = Command::new("mongodump");
    let with_default_options = dump_command
        .arg("--archive")
        .arg("--gzip")
        .arg(format!("--uri={}", dump_instruction.uri));

    let collection_targeted = {
        if dump_opts.collection.is_some() {
            dump_opts
                .collection
                .unwrap()
                .iter()
                .fold(with_default_options, |base, collection| {
                    base.arg(format!("--collection={}", collection))
                })
        } else {
            dump_instruction
                .excludes
                .iter()
                .fold(with_default_options, |base, exclude| {
                    base.arg(format!("--excludeCollection={}", exclude))
                })
        }
    };

    let mut restore_command = Command::new("mongorestore");
    let mongo_restore = restore_command
        .arg("--drop")
        .arg("-h")
        .arg(format!(
            "0.0.0.0:{}",
            if dump_opts.port.is_some() {
                dump_opts.port.unwrap()
            } else {
                dump_instruction.target_port
            }
        ))
        .arg("--gzip")
        .arg("--archive");

    println!(
        "{}\n{}\n\n{}\n{}",
        "dump command to be executed:".bright_blue().bold(),
        format!("{:?}", &collection_targeted).bold(),
        "restore command to be executed after dump command done:"
            .bright_blue()
            .bold(),
        format!("{:?}", &mongo_restore).bold()
    );

    let dump_child = collection_targeted
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("failed to execute mongodump command!:\n{}", e);
            exit(1)
        });

    let restored = mongo_restore
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(dump_child.stdout.unwrap_or_else(|| {
            eprintln!("failed to pipe mongodump stdout into mongorestore");
            exit(1)
        }))
        .output()
        .unwrap_or_else(|e| {
            eprintln!("failed to execute mongorestore:\n{}", e);
            exit(1)
        });

    println!("{}", String::from_utf8_lossy(&restored.stdout));
    eprintln!("{}", String::from_utf8_lossy(&restored.stderr));
}
