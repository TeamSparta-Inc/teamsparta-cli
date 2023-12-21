use crate::{cli::DumpCommand, config::MongoDumpInstruction};
use std::{
    collections::HashMap,
    process::{Command, Stdio},
    sync::Arc,
    thread, vec,
};
const MONGO_DUMP: &str = "mongodump";
const MONGO_RESTORE: &str = "mongorestore";

pub fn run_dump(
    dump_opts: DumpCommand,
    dump_service_config: HashMap<String, MongoDumpInstruction>,
) {
    let part_name = std::env::var("PART_NAME")
        .unwrap_or_else(|_| panic!("global env variable PART_NAME not set"));
    let dump_service_config = Arc::new(dump_service_config);

    let dump_instruction: MongoDumpInstruction = dump_service_config
        .get(&dump_opts.service)
        .unwrap_or_else(|| panic!("service {} not found in config", dump_opts.service))
        .clone();

    let collection_targeted_commands = {
        if let Some(family) = dump_opts.family {
            let target_family = dump_instruction
                .family
                .get(&family)
                .unwrap_or_else(|| panic!("family {} not found in config", family));

            target_family
                .iter()
                .map(|member| {
                    let mut dump_command = Command::new(MONGO_DUMP);

                    let with_default_options = dump_command
                        .arg("--archive")
                        .arg("--gzip")
                        .arg(format!("--uri={}", dump_instruction.source_uri))
                        .arg(format!("--db={}", dump_instruction.db_name));

                    with_default_options.arg(format!("--collection={}", member));

                    dump_command
                })
                .collect()
        } else if let Some(cols) = dump_opts.collections {
            cols.iter()
                .map(|col| {
                    let mut dump_command = Command::new(MONGO_DUMP);

                    let with_default_options = dump_command
                        .arg("--archive")
                        .arg("--gzip")
                        .arg(format!("--uri={}", dump_instruction.source_uri))
                        .arg(format!("--db={}", dump_instruction.db_name));

                    with_default_options.arg(format!("--collection={}", col));

                    dump_command
                })
                .collect()
        } else {
            let mut dump_command = Command::new(MONGO_DUMP);

            let with_default_options = dump_command
                .arg("--archive")
                .arg("--gzip")
                .arg(format!("--uri={}", dump_instruction.source_uri))
                .arg(format!("--db={}", dump_instruction.db_name));

            dump_instruction
                .excludes
                .iter()
                .fold(with_default_options, |base, exclude| {
                    base.arg(format!("--excludeCollection={}", exclude))
                });

            vec![dump_command]
        }
    };

    let mut handles = vec![];

    for mut collection_targeted_command in collection_targeted_commands {
        let part_name = part_name.clone();
        let dump_instruction = dump_instruction.clone();
        let handle = thread::spawn(move || {
            let mut restore_command = Command::new(MONGO_RESTORE);
            let mongo_restore = restore_command
                .arg("--drop")
                .arg(format!("--uri={}", &dump_instruction.target_uri))
                .arg(format!("--nsFrom={}.*", dump_instruction.db_name))
                .arg(format!(
                    "--nsTo={}_{}.*",
                    dump_instruction.db_name, part_name
                ))
                .arg("--gzip")
                .arg("--archive");
            let dump_child = collection_targeted_command
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to launch mongodump");
            let restore_child = mongo_restore
                .stdin(dump_child.stdout.expect("fail to open mongodump stdout"))
                .spawn()
                .expect("failed to launch mongorestore");

            let restored = restore_child
                .wait_with_output()
                .expect("failed to wait output");

            println!("{}", String::from_utf8_lossy(&restored.stdout));
            eprintln!("{}", String::from_utf8_lossy(&restored.stderr));
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
