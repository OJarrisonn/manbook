use std::{error::Error, path::Path, process::Command};

fn main() {
    println!("{:#?}", pages());
}

fn pages() -> Result<Vec<String>, Box<dyn Error>> {
    let out = Command::new("man")
        .arg("-w")
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8(out.stdout).unwrap();
    let dirs = stdout.split(":")
        .map(|s| Path::new(s.trim()))
        .map(Path::read_dir)
        .collect::<Result<Vec<_>, _>>()?;

    let subdirs = dirs.into_iter()
        .map(|dir| {
            dir.map(|entry| {
                entry.unwrap().path()
            })
            .map(|dir| {
                if dir.is_dir() {
                    dir.read_dir().unwrap().map(|entry| {
                        entry.unwrap().path()
                    }).collect::<Vec<_>>()
                } else {
                    vec![dir]
                }
            })
            .flatten()
        })
        .flatten();

    let pages = subdirs
        .map(|dir| {
            if dir.is_dir() {
                dir.read_dir().unwrap().map(|entry| {
                    entry.unwrap().path()
                }).collect::<Vec<_>>()
            } else {
                vec![dir]
            }
        })
        .flatten()
        .map(|dir| {
            dir.file_name().unwrap().to_str().map(|s| {
                if s.ends_with(".gz") {
                    s[..s.len()-3].to_string()
                } else {
                    s.to_string()
                }
            }).unwrap()
        })
        .collect();
    
    Ok(pages)
}