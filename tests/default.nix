let
  sources = import ../npins;
  pkgs = import sources.nixpkgs {
    config = { };
    overlays = [ ];
  };
  nix-heim = import ../.;
in
nix-heim pkgs [
  (
    {
      lib,
      pkgs,
      ...
    }:
    {
      overwrite = true;

      home = {
        directory = "/home/nixos";

        files = {
          "directory1" = {
            source = ./files/directory1;
            overwrite = false;
          };
          "directory2".source = ./files/directory2;
        };
      };

      xdg.config.files = {
        "foobar/foobar_1.txt".source = ./files/file;
        "foobar/foobar_2.txt".text = "foobar";
        "foobar/foobar_3.txt".text = lib.generators.toINI { } {
          main = {
            foo = "bar";
          };
        };
      };

      packages = [ pkgs.htop ];
    }
  )
]
