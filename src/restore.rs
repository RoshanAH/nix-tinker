use std::{
    fs, 
    io, 
    path::{self, Path, PathBuf},
    process::{self, Command},
};

use crate::{
    selection::Selection,
    unlink::NixLink,
    tmp::{hashed_dir, tmp_dir},
};

struct Link {
    symlink: PathBuf,
    tmpdir: PathBuf,
    nix_store_file: PathBuf,
}

impl Link {
    fn from(nix_link: NixLink, dir: PathBuf) -> Self {
        Link {
            symlink: nix_link.symlink,
            tmpdir: dir,
            nix_store_file: nix_link.nix_store_file,
        }
    }
}

impl From<NixLink> for Link {
    fn from(link: NixLink) -> Self {
        let dir = hashed_dir(&link.symlink);
        Link::from(link, dir)
    }
}


#[derive(Debug)]
struct Error {
    path: PathBuf,
    stderr: bool,
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    NotExisting,
    NotLink,
    NotTinker,
    IO(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::NotExisting => write!(f, "{} does not exist", self.path.display()),
            ErrorKind::NotLink => write!(f, "{} is not a link", self.path.display()),
            ErrorKind::NotTinker => write!(f, "{} is not managed by nix tinker", self.path.display()),
            ErrorKind::IO(why) => write!(f, "io error: {}", why),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let ErrorKind::IO(why) = &self.kind {
            Some(why)
        } else {
            None
        }
    }
}

fn read_link(path: PathBuf) -> Result<Link, Error> {

    if !path.exists() {
        return Err(Error {
            path,
            stderr: false,
            kind: ErrorKind::NotExisting,
        });
    }

    if !path.is_symlink() {
        return Err(Error {
            path,
            stderr: false,
            kind: ErrorKind::NotLink,
        });
    }

    let realpath = fs::canonicalize(&path).map_err(|why| Error {
        path: path.clone(),
        stderr: true,
        kind:  ErrorKind::IO(why),
    })?;

    if !realpath.starts_with("/tmp/nix-tinker/") {
        return Err(Error {
            path,
            stderr: false,
            kind: ErrorKind::NotTinker,
        });
    }

    let abspath = path::absolute(&path).map_err(|why| Error {
        path: path.clone(),
        stderr: true,
        kind: ErrorKind::IO(why)
    })?;

    let dir = hashed_dir(&abspath);

    let link = Link::from(toml::from_str(
        &fs::read_to_string(dir.join("link.toml")).unwrap()
    ).map_err(|why| Error {
        path: path.clone(),
        stderr: true,
        kind: ErrorKind::IO(io::Error::new(io::ErrorKind::Other, why)),
    })?, dir);

    Ok(link)
}

fn restore_file(link: &Link) -> io::Result<()> {

    let check_command = |result: Result<(), process::ExitStatusError>| {
        result.map_err(|why| {
            use io::{Error, ErrorKind::Other};
            Error::new(Other, why)
        })
    };

    check_command(Command::new("unlink")
        .arg(&link.symlink)
        .status()?.exit_ok())?;

    check_command(Command::new("ln")
        .arg("-s")
        .arg(&link.nix_store_file)
        .arg(&link.symlink)
        .status()?.exit_ok())?;

    check_command(Command::new("rm")
        .arg("-r")
        .arg(&link.tmpdir)
        .status()?.exit_ok())?;

    Ok(())
}


pub fn restore(selection: Selection) {
    for read in selection.into_iter().map(|file| read_link(file)) {
        match read {
            Err(why) => {
                let msg = format!("error restoring {}: {}", why.path.display(), why);
                if why.stderr {
                    eprintln!("{msg}");
                } else {
                    println!("{msg}");
                }
            },

            Ok(link) => {
                match restore_file(&link) {
                    Err(why) => {
                        eprintln!("unable to restore {}: {}", link.symlink.display(), why);
                    }
                    Ok(_) => {
                        println!("restored {}", link.symlink.display());
                    },
                }
            }
        }
    }
}

pub fn restore_all() {
    let restores = tmp_dir()
        .read_dir()
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|result| {
            let dir = result.ok()?.path();
            let link = toml::from_str(&fs::read_to_string(dir.join("link.toml")).unwrap()).ok()?;
            let link = Link::from(link, dir);

            if !fs::canonicalize(&link.symlink).ok()?.starts_with(&link.tmpdir) {
                let _ = Command::new("rm")
                    .arg("-r")
                    .arg(&link.tmpdir)
                    .status();
                return None;
            }

            let restore = restore_file(&link);
            Some((link.symlink, restore))
        });

    for restore in restores {
        match restore {
            (path, Err(why)) => {
                eprintln!("error restoring {}: {}", path.display(), why);
            },
            (path, Ok(())) => {
                println!("restored {}", path.display());
            },
        }

    }
}
