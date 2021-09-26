// TODO: multi-threading

use ignore::Walk;
use json;
use lazy_static::lazy_static;
use std::env;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::Path;

type JVal = json::JsonValue;

lazy_static! {
    static ref LANGS: JVal = json::parse(include_str!("langs.json")).unwrap();
}

fn derive_type(
    file: &Path,
    contents: &String,
    by_ext: &JVal,
    by_file: &JVal,
    by_shebang: &JVal,
) -> Option<String> {
    if let Some(ty) = file.extension().and_then(OsStr::to_str).map(|e| &by_ext[e]) {
        return ty.as_str().map(String::from);
    } else if let Some(ty) = Path::file_name(file)
        .and_then(|e| e.to_str())
        .map(|e| &by_file[e])
    {
        return ty.as_str().map(String::from);
    } else if let Some(ty) = contents.split("\n").next().and_then(|e| {
        e.split(" ")
            .next()
            .and_then(|e| e.split("/").last().map(|e| &by_shebang[e]))
    }) {
        return ty.as_str().map(String::from);
    }
    return None;
}

fn files_to_json(files: &Vec<(String, usize, String)>) -> JVal {
    let mut j = JVal::new_object();
    for file in files {
        j[file.0.as_str()] = json::object! {
            "lines" => file.1,
            "type" => file.2.as_str(),
        };
    }
    return j;
}

fn main() {
    let mut json = false;
    let mut by_file = false;
    let mut dirs = Vec::new();
    for arg in env::args() {
        if arg == "--json" {
            json = !json;
        } else if arg == "--by-file" {
            by_file = !by_file;
        } else {
            dirs.push(arg);
        }
    }
    if dirs.len() == 0 {
        dirs.push(String::from("."));
    }
    // TODO: use dirs
    let by_ext = &LANGS["by_ext"];
    let by_file = &LANGS["by_file"];
    let by_shebang = &LANGS["by_shebang"];
    let raw_files = Walk::new(".")
        .filter_map(Result::ok)
        .filter(|e| e.file_type().unwrap().is_file());
    let files: Vec<(String, usize, String)> = raw_files
        .map(|raw_file| {
            let contents = match read_to_string(raw_file.path()) {
                Ok(s) => s,
                Err(_) => return None,
            };
            let path = String::from(raw_file.path().to_str().unwrap());
            // TODO: make sense
            let len = contents.lines().count();
            if let Some(ty) = derive_type(raw_file.path(), &contents, by_ext, by_file, by_shebang) {
                return Some((path, len, ty));
            }
            return None;
        })
        .filter(|e| e.is_some())
        .map(|e| e.unwrap())
        .collect();
    json = true;
    if json {
        // TODO: "header"
        // TODO: cloc compatibility
        println!("{}", files_to_json(&files).dump());
    } else {
        // TODO: support the table
        panic!("this is not supported at the moment");
    }
}
