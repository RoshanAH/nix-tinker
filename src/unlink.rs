use std::{
    fs::create_dir_all, 
    path::PathBuf
};
use crate::{
    Selection, 
    tmp::hash_link_dir
};


struct NixLink {
    link: PathBuf,
    store_file: PathBuf,
}

enum NixLinkRead {
    NotExisting(PathBuf), // path does not exist
    NotLink(PathBuf), // path is not symlink
    NotNix(PathBuf), // path is not a nix link
    Ok(NixLink), // path is a nix link
}

fn read_nix_store_file(file: PathBuf) -> NixLinkRead {

    if !file.exists() {
        return NixLinkRead::NotExisting(file);
    }

    match file.read_link() {
        Err(_) => NixLinkRead::NotLink(file),
        Ok(linked_file) => {
            if !linked_file.starts_with("/nix/store/") {
                NixLinkRead::NotNix(file)
            } else {
                NixLinkRead::Ok(NixLink {
                    link: file,
                    store_file: linked_file,
                })
            }
        },
    }
}

fn unlink_nix_link(nix_link: &NixLink) -> std::io::Result<()>{
    let dir = hash_link_dir(&nix_link.link);
    create_dir_all(&dir)?;
    let (link_path, store_path) = (std::path::absolute(&nix_link.link), todo!());
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
            NixLinkRead::Ok(link) => {
                match unlink_nix_link(&link) {
                    Ok(_) => {
                        println!("unlinked {}", link.link.display())
                    },
                    Err(err) => {
                        eprintln!("unable to unlink {}: {}", link.link.display(), err)
                    }
                }
            },
        }
    }
}

