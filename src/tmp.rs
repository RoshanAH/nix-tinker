use std::fs::create_dir_all;
use std::io;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};



// hashes the filename
fn hash<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}


// returns the pathbuf of tmpdir, dir does not have to exist
pub fn tmp_dir() -> PathBuf {
    PathBuf::from("/tmp/nix-tinker/")
}

// returns the pathbuf of tmpdir and creates it if it does not exist
fn get_tmp_dir() -> io::Result<PathBuf> {
    let dir = tmp_dir();
    if !dir.exists() {
        create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn hashed_dir<P: AsRef<Path>>(path: P) -> PathBuf{
    let hash = PathBuf::from(hash(path));
    let mut ret = tmp_dir();
    ret.push(hash);
    ret
}
