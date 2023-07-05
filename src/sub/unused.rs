use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs::{read_to_string, remove_file},
    path::PathBuf,
};
use walkdir::WalkDir;

use crate::cli::UnusedCommand;
fn match_asset_ext(ext: &str) -> bool {
    ext == "png"
        || ext == "jpg"
        || ext == "ico"
        || ext == "svg"
        || ext == "mp3"
        || ext == "jpeg"
        || ext == "xml"
}

fn match_code_ext(ext: &str) -> bool {
    ext == "ts" || ext == "js" || ext == "css" || ext == "tsx" || ext == "jsx" || ext == "scss"
}
fn contains_asset_ext(s: &str) -> bool {
    s.contains("png")
        || s.contains("jpg")
        || s.contains("ico")
        || s.contains("svg")
        || s.contains("mp3")
        || s.contains("jpeg")
        || s.contains("xml")
}

fn make_asset_set(
    asset_dir: &PathBuf,
    asset_set: &mut HashSet<Box<OsString>>,
    asset_map: &mut HashMap<Box<OsString>, PathBuf>,
) {
    let entries = WalkDir::new(asset_dir)
        .into_iter()
        .filter_map(|entry| entry.ok());

    for entry in entries {
        let file_type = entry.file_type();

        if file_type.is_file() {
            let ext = entry.path().extension().unwrap().to_string_lossy();
            if match_asset_ext(&ext) {
                let file_name = Box::new(entry.file_name().to_owned());
                asset_map.insert(file_name.clone(), entry.into_path());
                asset_set.insert(file_name);
            }
        }
    }
}

fn search_assets(target_dir: &PathBuf, asset_set: &mut HashSet<Box<OsString>>) {
    let entries = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|entry| entry.ok());

    for entry in entries {
        let file_type = entry.file_type();

        if file_type.is_file() {
            let ext = entry.path().extension().unwrap().to_string_lossy();
            if match_code_ext(&ext) {
                let file_content = read_to_string(entry.path()).unwrap_or(String::new());
                if contains_asset_ext(&file_content) {
                    for key in asset_set.clone().into_iter() {
                        if file_content.contains(key.to_str().unwrap()) {
                            asset_set.remove(&key);
                        }
                    }
                }
            }
        }
    }
}

pub fn run_unused(unused_opts: UnusedCommand) {
    let mut asset_set: HashSet<Box<OsString>> = HashSet::new();
    let mut asset_map: HashMap<Box<OsString>, PathBuf> = HashMap::new();

    for asset_dir in &unused_opts.asset_dirs {
        make_asset_set(asset_dir, &mut asset_set, &mut asset_map);
    }

    let all_asset_len = asset_set.len();

    for target_dir in &unused_opts.target_dirs {
        search_assets(target_dir, &mut asset_set);
    }

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
        println!("{:#?}\n{}", asset_set, asset_set.len());
    }
}
