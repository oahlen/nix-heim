# Nix Heim

> The realm of your Nix based dotfiles

Nix Heim is a light-weight alternative to [home-manager](https://github.com/nix-community/home-manager) that manages your dotfiles and user programs using Nix.
It uses a custom file linker written in Rust to perform linking of files after a user profile has been activated.
Heim does not expose a module system like home-manager instead opting to give you the fundamental configuration building blocks including:

* Specify packages to install for your user environment, similar to home-manager, nix profile etc.
* Link existing dotfiles to desired location in your home directory.
* Create dotfiles using nix expressions giving you the option to build your own home-manager like modules.
* Create variants of the same dotfile, Heim allows you create multiple variants of a file that can be instantly switched out using the switch command, all without performing a new rebuild.
* Quality of life options like session variables that can include nix expression easily sourced in your shell RC files.
* Activation hooks: optional shell snippets to always run when performing profile switching.
* Super fast activation: Heim is very fast in building and applying the dotfiles manifest using as few file-system operations as possible.

## Getting Started

The best way to get started with Nix Heim is to install it as a standalone nix profile for your user.
Heim doesn't impose its own version of nixpkgs but encourages you use your own for efficiency reasons.

A simple setup (`default.nix`) using npins looks as follows:

```nix
let
  pins = import ./npins;
  pkgs = import pins.nixpkgs { };
  heim = import pins.nix-heim;
in
heim pkgs {
  modules = [
    (
      {
        config,
        lib,
        pkgs,
        ...
      }:
      {
        user = "nixos";

        home = {
          files = {
            "myfile".source = ./myfile;
          };

          packages = [ pkgs.htop ];
        };
      }
    )
  ];
}
```

To perform first installation run:

```bash
nix run -f default.nix "install"
```

After first installation the `heim-switch` script should be available on your path, subsequent rebuilds can be performed by running:

```bash
heim-switch default.nix
```
