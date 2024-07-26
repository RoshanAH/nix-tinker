use std::fs::create_dir_all;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

// hashes the filename
fn hash<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let path = std::path::absolute(path.as_ref())?;
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}


// returns the pathbuf of tmpdir, dir does not have to exist
fn tmp_dir() -> PathBuf {
    PathBuf::from("/tmp/nix-tinker/")
}

// returns the pathbuf of tmpdir and creates it if it does not exist
fn get_tmp_dir() -> std::io::Result<PathBuf> {
    let dir = tmp_dir();
    if !dir.exists() {
        create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn hash_link_dir<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
    let hash = PathBuf::from(hash(path)?);
    let mut ret = tmp_dir();
    ret.push(hash);
    Ok(ret)
}
