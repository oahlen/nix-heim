let
  sources = import ../npins;
  pkgs = import sources.nixpkgs { };
  nix-heim = import ../.;
in
nix-heim pkgs [
  {
    home = {
      directory = "/home/nixos";

      files = {
        "directory1".source = ./files/directory1;
        "directory2" = {
          source = ./files/directory2;
          recursive = true;
        };
      };
    };

    xdg.config.files = {
      "foobar/foobar_1.txt".source = ./files/file;
      "foobar/foobar_2.txt".text = "foobar";
    };
  }
]
