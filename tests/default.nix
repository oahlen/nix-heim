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
      config,
      lib,
      pkgs,
      ...
    }:
    {
      user = "nixos";
      overwrite = true;

      home = {
        files = {
          # Test expanded directory works
          "directory1".source = ./files/directory1;

          # Test inverted overwrite works and propagates to child entries
          "directory2" = {
            source = ./files/directory2;
            overwrite = false;
          };

          # Test derivation directory is symlinked as a single entry (no recursion)
          "derivation-dir".source = pkgs.runCommand "test-derivation-dir" { } ''
            mkdir -p $out
            echo "hello" > $out/hello.txt
          '';
        };

        packages = [ pkgs.htop ];

        sessionVariables = {
          EDITOR = "vim";
          FILE = ./default.nix;
          HTOP_PATH = lib.getExe pkgs.htop;
          INTEGER = 1;
          PAGER = "less";
          PATH = "$HOME/bin";
        };

        sessionPath = [
          "$HOME/.local/bin"
          "${config.xdg.config.directory}/scrips"
        ];
      };

      xdg.config.files = {
        # Test file with source works
        "foobar/foobar_1.txt".source = ./files/file_1;

        # Test file with 2 variants works
        "foobar/foobar_2.txt" = {
          variants = {
            dark = {
              source = ./files/file_1;
              default = true;
            };

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
    }
  )
]
