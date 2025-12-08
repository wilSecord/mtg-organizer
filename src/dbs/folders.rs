use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::APPNAME_DIRECTORY;

pub fn save_directory() -> Option<PathBuf> {
    let data_dir = std::env::var_os("XDG_DATA_HOME")
        .map(|x| PathBuf::from(x))
        .or_else(|| std::env::home_dir().map(|x| x.join(".local/share")))
        .map(|x| x.join(APPNAME_DIRECTORY))?;

    match fs::create_dir_all(&data_dir) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        _ => return None,
    }

    return Some(data_dir);
}
