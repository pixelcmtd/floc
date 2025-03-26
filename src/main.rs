// TODO: more multi-threading

use clap::Parser;
use ignore::Walk;
use json;
use lazy_static::lazy_static;
use rayon::iter::*;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::Path;

type JVal = json::JsonValue;

lazy_static! {
    static ref LANGS_RAW: JVal = json::parse(include_str!("langs.json")).unwrap();
    static ref BY_EXT: &'static JVal = &LANGS_RAW["by_ext"];
    static ref BY_FILE: &'static JVal = &LANGS_RAW["by_file"];
    static ref BY_SHEBANG: &'static JVal = &LANGS_RAW["by_shebang"];
}

#[derive(Parser)]
#[clap(name = "floc")]
struct Opt {
    #[clap(long)]
    json: bool,

    #[clap(long)]
    by_file: bool,

    // TODO: yaml, short options
    #[clap(name = "DIR", default_value = ".")]
    dirs: Vec<String>,
}

fn derive_type(file: &Path, contents: &String) -> Option<String> {
    if let Some(ty) = file.extension().and_then(OsStr::to_str).map(|e| &BY_EXT[e]) {
        return ty.as_str().map(String::from);
    } else if let Some(ty) = Path::file_name(file)
        .and_then(|e| e.to_str())
        .map(|e| &BY_FILE[e])
    {
        return ty.as_str().map(String::from);
    } else if let Some(ty) = contents.split("\n").next().and_then(|e| {
        // TODO: check if this works (it doesnt seem to)
        e.split(" ")
            .next()
            .and_then(|e| e.split("/").last().map(|e| &BY_SHEBANG[e]))
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
    j
}

fn main() {
    let config = Opt::parse();
    let raw_files = config
        .dirs
        .into_iter()
        .map(Walk::new)
        .map(|walk| {
            walk.filter_map(Result::ok)
                .filter(|e| e.file_type().unwrap().is_file())
        })
        .flatten();
    let files: Vec<(String, usize, String)> = raw_files
        .par_bridge()
        .map(|raw_file| {
            let contents = read_to_string(raw_file.path()).ok()?;
            let path = String::from(raw_file.path().to_str().unwrap());
            // TODO: make sense
            let len = contents.lines().count();
            derive_type(raw_file.path(), &contents).map(|ty| (path, len, ty))
        })
        .filter(|e| e.is_some())
        .map(|e| e.unwrap())
        .collect();
    if config.json || true {
        // TODO: "header"
        // TODO: cloc compatibility
        println!("{}", files_to_json(&files).dump());
    } else {
        // TODO: support the table
        panic!("this is not supported at the moment");
    }
}
