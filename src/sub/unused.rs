use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs::{read_to_string, remove_file},
    path::PathBuf,
};
use walkdir::WalkDir;

use crate::cli::UnusedCommand;
fn is_asset(str_file_name: Option<&str>) -> bool {
    str_file_name
        .map(|s| {
            let len = s.len();
            if len >= 5 {
                let last_3 = s.get(len - 3..).unwrap_or("");
                let last_4 = s.get(len - 4..).unwrap_or("");

                !last_3.is_empty()
                    && (last_3 == "png"
                        || last_3 == "jpg"
                        || last_3 == "ico"
                        || last_3 == "svg"
                        || last_3 == "mp3"
                        || last_4 == "jpeg")
            } else {
                false
            }
        })
        .unwrap_or(false)
}

fn make_asset_set(
    asset_dir: &PathBuf,
    search_depth: &i64,
    asset_set: &mut HashSet<Box<OsString>>,
    asset_map: &mut HashMap<Box<OsString>, PathBuf>,
) {
    let entries = WalkDir::new(asset_dir)
        .min_depth(1)
        .max_depth(search_depth.to_owned() as usize)
        .into_iter()
        .filter_map(|entry| entry.ok());

    for entry in entries {
        let file_type = entry.file_type();
        let str_file_name = entry.file_name().to_str();
        let is_hidden = str_file_name.map(|s| s.starts_with('.')).unwrap_or(false);

        if file_type.is_file() && !is_hidden && is_asset(str_file_name) {
            let file_name = Box::new(entry.file_name().to_owned());
            asset_map.insert(file_name.clone(), entry.into_path());
            asset_set.insert(file_name);
        }
    }
}

fn search_assets(target_dir: &PathBuf, search_depth: &i64, asset_set: &mut HashSet<Box<OsString>>) {
    let entries = WalkDir::new(target_dir)
        .min_depth(1)
        .max_depth(search_depth.to_owned() as usize)
        .into_iter()
        .filter_map(|entry| entry.ok());

    for entry in entries {
        let file_type = entry.file_type();
        let str_file_name = entry.file_name().to_str();
        let is_hidden = str_file_name.map(|s| s.starts_with('.')).unwrap_or(false);

        if file_type.is_file() && !is_hidden && !is_asset(str_file_name) {
            let file_content = read_to_string(entry.path()).unwrap_or(String::new());
            if file_content.is_empty() {
                continue;
            }

            let asset_clone = asset_set.clone();
            let search_keys = asset_clone.iter();

            for key in search_keys {
                if file_content.contains(key.to_str().unwrap()) {
                    asset_set.remove(key);
                }
            }
        }
    }
}

pub fn run_unused(unused_opts: UnusedCommand) {
    let mut asset_set: HashSet<Box<OsString>> = HashSet::new();
    let mut asset_map: HashMap<Box<OsString>, PathBuf> = HashMap::new();

    make_asset_set(
        &unused_opts.asset_dir,
        &unused_opts.asset_depth,
        &mut asset_set,
        &mut asset_map,
    );

    let all_asset_len = asset_set.len();

    search_assets(
        &unused_opts.target_dir,
        &unused_opts.target_depth,
        &mut asset_set,
    );

    let unused_len = asset_set.len();

    if unused_opts.delete {
        for file_name in &asset_set {
            let del_path = asset_map.get(file_name).unwrap();
            println!("{:?}", del_path);
            remove_file(del_path).unwrap();
        }

        println!(
            "unused_assets: {:#?}\ndeleted count: {}\nremaining count: {}",
            &asset_set,
            unused_len,
            all_asset_len - unused_len
        );
    } else {
        println!("{:#?}", asset_set);
    }
}
