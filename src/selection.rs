use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};
use walkdir::{DirEntry, WalkDir};


#[derive(Args)]
/// File Operation Args
pub struct Selection {
    /// The files/directories to target
    paths: Vec<PathBuf>,
    #[arg(short)]
    /// Select files in the specified directory recursively
    recursive: bool,
}

impl IntoIterator for Selection {
    type Item = PathBuf;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    // iterator of all the nix store symlinks
    fn into_iter(self) -> Self::IntoIter {

        let iter = self.paths.into_iter();
        let (dirs, other): (Vec<_>, Vec<_>) = iter.partition(|path| path.is_dir());
        let (dirs, other) = (dirs.into_iter(), other.into_iter());
        let walk = move |dir: PathBuf| {
            let filter = |result: Result<DirEntry, _>| {
                if let Ok(dir_entry) = result {
                    Some(dir_entry.into_path())
                } else {
                    None
                }
            };
            if self.recursive {
                WalkDir::new(dir).into_iter().filter_map(filter)
            } else {
                WalkDir::new(dir).max_depth(1).into_iter().filter_map(filter)
            }
        };

        // TODO remove duplicates
        Box::new(other.chain(dirs.flat_map(walk)))
    }
}
