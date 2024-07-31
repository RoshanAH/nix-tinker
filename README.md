<h1 align="center">nix-tinker</h1>

## What is it?

nix-tinker is a utility that allows the user to tinker with various config files managed by nix without having to rebuild to apply changes. 
One of the downsides of using home-manager/nixos is that for each config change to be reflected on your machine, you have to rebuild. 
This gets tedious for configs that need to be rapidly tweaked such as hyprland or nvim.
nix-tinker temporarily *unlink*s specified files linked to the nix store and links the files instead to temporary copies that you have write access to.
This way, you are able to freely tinker without rebuilding.
Once you are satisfyed with your tinkering, you can apply your desired changes to your nix config, *restore* your link to the nix store, and rebuild.

## Installation

Install nix-tinker as you would a regular flake.

```nix
# flake.nix
{

  inputs = {
    # ...
    nix-tinker.url = "github:RoshanAH/nix-tinker"; # include nix-tinker as an input
  };

  outputs = { nixpkgs, ... }@inputs: {
    nixosConfigurations.default = nixpkgs.lib.nixosSystem {
      specialArgs = {inherit inputs;};
      modules = [
        ./configuration.nix
      ];
    };
  };
  
}
```
```nix
# configuration.nix
{inputs, pkgs, ...}: {
    # ...
    environment.systemPackages = (with pkgs; [
        # ...
    ]) ++ [
        inputs.nix-tinker.packages.${pkgs.stdenv.hostPlatform.system}.default
    ]; 
    # ...
}

```

## Usage

```
$ nix-tinker help
Usage: nix-tinker [OPTIONS] <COMMAND>

Commands:
  unlink       Unlinks files from the nix store
  restore      Restores unlinked files from the nix store
  restore-all  Restores all unlinked files from the nix store
  help         Print this message or the help of the given subcommand(s)

Options:
      --dry-run  Preview files that will be changed
  -h, --help     Print help
  -V, --version  Print version
```

## Hacking

Just `nix develop`.
