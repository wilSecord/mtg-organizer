use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    process::{Command, ExitStatus},
};

use serde_json::{self, Map};

pub fn run_command(cmd: &str) {
    let status = Command::new("/bin/sh").arg("-c").arg(cmd).status().unwrap();

    assert!(status.success());
}

pub fn main() {
    if !fs::exists("data/AllSetFiles.tar.xz").unwrap() {
        run_command(
            "curl -o data/AllSetFiles.tar.xz 'https://mtgjson.com/api/v5/AllSetFiles.tar.xz'",
        );
    }
    //creates data/AllSetFiles/*.json
    if !fs::exists("data/AllSetFiles").unwrap() {
        run_command("tar --xz -xf data/AllSetFiles.tar.xz");
    }

    let mut results = Vec::new();

    for file in fs::read_dir("data/AllSetFiles").unwrap() {
        let file = file.unwrap();

        eprintln!(
            "Processing set {}...",
            file.file_name().into_string().unwrap()
        );

        let json: serde_json::Value =
            serde_json::from_reader(BufReader::new(File::open(file.path()).unwrap())).unwrap();

        let mut sets = Map::new();

        let set_code = json["data"]["code"].as_str().unwrap().to_string();

        let cardname_keyvals = json["data"]["cards"]
            .as_array()
            .unwrap()
            .iter()
            .map(|card| {
                serde_json::Value::Array(vec![card["name"].to_owned(), card["number"].to_owned()])
            })
            .collect();

        sets.insert(set_code.clone(), serde_json::Value::Array(cardname_keyvals));

        if let serde_json::Value::String(token_set_code) = &json["data"]["tokenSetCode"] {
            let tokenname_iter = json["data"]["tokens"]
                .as_array()
                .unwrap()
                .iter()
                .map(|card| {
                    serde_json::Value::Array(vec![
                        card["name"].to_owned(),
                        card["number"].to_owned(),
                    ])
                });

            if *token_set_code == set_code {
                sets.get_mut(&set_code)
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .extend(tokenname_iter);
            } else {
                sets.insert(
                    token_set_code.to_owned(),
                    serde_json::Value::Array(tokenname_iter.collect()),
                );
            }
        }

        results.push(serde_json::Value::Object(sets));
    }

    let output = fs::File::create("data/fullsets.json")
        .map(BufWriter::new)
        .unwrap();

    serde_json::to_writer_pretty(output, &results).unwrap();
}
