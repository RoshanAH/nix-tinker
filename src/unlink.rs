use std::{
    fs, io,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{self, Command},
};
use serde::{ 
    Serialize,
    Deserialize,
};
use crate::{
    Selection, 
    tmp::hashed_dir,
};

#[derive(Serialize, Deserialize)]
pub struct NixLink {
    pub symlink: PathBuf,
    pub nix_store_file: PathBuf,
}

impl NixLink {
    fn from<P: AsRef<Path>>(link: P, store_file: P) -> io::Result<Self> { 
        use std::path::absolute as abs;
        Ok(Self {
            symlink: abs(link)?,
            nix_store_file: abs(store_file)?,
        })
    }
}

enum NixLinkRead {
    NotExisting(PathBuf), // path does not exist
    NotLink(PathBuf), // path is not symlink
    NotNix(PathBuf), // path is not a nix link
    IoErr { // io error
        path: PathBuf, 
        why: io::Error
    }, 
    Ok(NixLink), // path is a nix link
}

fn read_nix_store_file(file: PathBuf) -> NixLinkRead {

    if !file.exists() {
        return NixLinkRead::NotExisting(file);
    }

    if !file.is_symlink() {
        return NixLinkRead::NotLink(file);
    }

    match file.read_link() {
        Err(why) => NixLinkRead::IoErr {
            path: file,
            why,
        },
        Ok(linked_file) => {
            if !linked_file.starts_with("/nix/store/") {
                NixLinkRead::NotNix(file)
            } else {
                match NixLink::from(&file, &linked_file) {
                    Err(why) => NixLinkRead::IoErr {
                        path: file,
                        why
                    },
                    Ok(ret) => NixLinkRead::Ok(ret),
                }
            }
        },
    }
}

fn unlink_nix_link(nix_link: &NixLink) -> io::Result<()>{

    let dir = hashed_dir(&nix_link.symlink);
    let rmdir = || {
            if dir.exists() {
                Command::new("rm")
                    .arg("-r")
                    .arg(&dir)
                    .status().unwrap();
            }
    };
    let check_command = |result: Result<(), process::ExitStatusError>| {
        result.map_err(|why| {
            rmdir();
            use io::{Error, ErrorKind::Other};
            Error::new(Other, why)
        })
    };

    if dir.exists() {
        Command::new("rm")
            .arg("-r")
            .arg(&dir)
            .status()?;
    }

    fs::create_dir_all(&dir)?;
    let serialized = toml::to_string(&nix_link).map_err(|why| {
        use io::{Error, ErrorKind};
        Error::new(ErrorKind::Other, why)
    })?;
    let mut link_file = fs::File::create(&dir.join("link.toml"))?;
    link_file.write_all(serialized.as_bytes())?;
    let copy = dir.join(nix_link.symlink.file_name().unwrap());

    check_command(Command::new("unlink")
        .arg(&nix_link.symlink)
        .status()?.exit_ok())?;

    check_command(Command::new("cp")
        .arg("--no-preserve=mode")
        .arg("-r")
        .arg(fs::canonicalize(&nix_link.nix_store_file)?)
        .arg(&copy)
        .status()?.exit_ok())?;

    check_command(Command::new("ln")
        .arg("-s")
        .arg(&copy)
        .arg(&nix_link.symlink)
        .status()?.exit_ok())?;

    Ok(())
}

pub fn unlink(files: Selection) {
    let files = files.into_iter().map(|file| read_nix_store_file(file));
    for file in files {
        match file {
            NixLinkRead::NotExisting(file) => {
                println!("unable to unlink {}: file does not exist", file.display())
            },
            NixLinkRead::NotLink(file) => {
                println!("unable to unlink {}: file is not a symlink", file.display())
            },
            NixLinkRead::NotNix(file) => {
                println!("unable to unlink {}: symlink does not point to nix store", file.display())
            },
            NixLinkRead::IoErr { path: file, why }=> {
                eprintln!("unable to unlink {}: {}", file.display(), why)
            },
            NixLinkRead::Ok(link) => {
                match unlink_nix_link(&link) {
                    Ok(_) => {
                        println!("unlinked {}", link.symlink.display())
                    },
                    Err(why) => {
                        eprintln!("unable to unlink {}: {}", link.symlink.display(), why)
                    }
                }
            },
        }
    }
}

