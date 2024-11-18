use std::{error::Error, path::PathBuf, process::Command};

use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

fn main() -> Result<(), Box<dyn Error>> {
    let dirs = mandirs()?;
    let pages = dirs.into_par_iter()
        .map(dir_read_all)
        .collect::<Result<Vec<_>, _>>().unwrap() // TODO: handle error
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let pages = pages_from_files(pages);

    println!("{:#?}", pages);

    Ok(())
}

/// Gets the list of directories where to look for manpages
fn mandirs() -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let out = Command::new("man")
        .arg("-w")
        .output()?;

    let stdout = String::from_utf8(out.stdout)?;
    let dirs = stdout.split(":")
        .map(|s| PathBuf::from(s.trim()))
        .collect();

    Ok(dirs)
}

/// Given a directory, reads all files in it returning a partitioned iterator of files and subdirectories
/// 
/// The first element of the tuple is a vector of subdirectories and the second element is a vector of files
fn dir_read_open(dir: PathBuf) -> Result<(Vec<PathBuf>, Vec<PathBuf>), Box<dyn Error + Send>> {
    let (dirs, files) = dir.read_dir().map_err(|e| Box::new(e) as Box<dyn Error + Send>)?
        .par_bridge()
        .map(|entry| entry.unwrap().path())
        .partition::<Vec<_>, Vec<_>, _>(|entry| entry.is_dir());

    Ok((dirs, files))
}

/// Given a directory, returns a list of the files in it and its subdirectories
fn dir_read_all(dir: PathBuf) -> Result<Vec<PathBuf>, Box<dyn Error + Send>> {
    dir_read_open(dir).map(|(dirs, files)| {
        if dirs.len() == 0 {
            files
        } else {
            dirs.into_par_iter()
                .map(dir_read_all)
                .collect::<Result<Vec<_>, _>>().unwrap() // TODO: handle error
                .into_iter()
                .flatten()
                .chain(files.into_iter())
                .collect()
        }
    })
}

/// Receives a list of paths that are files and returns a list of their names without the `.gz` extension
fn pages_from_files(files: Vec<PathBuf>) -> Vec<String> {
    files.into_par_iter()
        .map(|file| {
            file.file_name().unwrap().to_str().map(|s| {
                if s.ends_with(".gz") {
                    s[..s.len()-3].to_string()
                } else {
                    s.to_string()
                }
            }).unwrap()
        })
        .collect::<Vec<_>>()
}