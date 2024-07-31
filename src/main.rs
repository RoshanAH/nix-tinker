#![feature(exit_status_error)]
mod unlink;
mod restore;
mod selection;
mod tmp;

use clap::{Parser, Subcommand};

use crate::{
    selection::Selection,
    unlink::unlink,
    restore::{restore, restore_all},
};

#[derive(Parser)]
#[command(name = "nix tinker", version, about, long_about = None)]
struct Cli {

    #[arg(long, global = true)]
    /// Preview files that will be changed
    dry_run: bool, // TODO implement this

    #[command(subcommand)]
    command: Commands,
}



#[derive(Subcommand)]
enum Commands {
    /// Unlinks files from the nix store
    Unlink(Selection),
    /// Restores unlinked files from the nix store
    Restore(Selection),
    /// Restores all unlinked files from the nix store
    RestoreAll,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Unlink(selection) => {
            unlink(selection);
        },

        Commands::Restore(selection)=> { 
            restore(selection);
        },

        Commands::RestoreAll => {
            restore_all();
        },
    }
}
