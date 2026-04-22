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
          # Test expanded directory works
          "directory1".source = ./files/directory1;

          # Test inverted overwrite works and propagates to child entries
          "directory2" = {
            source = ./files/directory2;
            overwrite = false;
          };
        };
      };

      xdg.config.files = {
        # Test file with source works
        "foobar/foobar_1.txt".source = ./files/file_1;

        # Test file with 2 variants works
        "foobar/foobar_2.txt" = {
          source = ./files/file_1;
          variants = {
            dark.source = ./files/file_1;
            light.text = ''
              Content
            '';
          };
        };

        # Test file with generator works
        "foobar/foobar_3.txt".text = lib.generators.toINI { } {
          main = {
            foo = "bar";
          };
        };
      };

      sessionVariables = {
        EDITOR = "vim";
        PAGER = "less";
        HTOP_PATH = lib.getExe pkgs.htop;
      };

      packages = [ pkgs.htop ];
    }
  )
]
