let
  sources = import ../npins;
  pkgs = import sources.nixpkgs {
    config = { };
    overlays = [ ];
  };
  nix-heim = import ../.;
in
nix-heim pkgs {
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
        overwrite = true;

        home = {
          files = {
            # Test directory is symlinked as-is by default, copied into the Nix store
            "directory1".source = ./files/directory1;

            # Test recursive directory expansion works and overwrite propagates to child entries correctly
            "directory2" = {
              source = ./files/directory2;
              overwrite = false;
              recursive = true;
            };

            # Test keepOutOfStore preserves the original path instead of copying it into the store
            "directory3" = {
              source = ./files/directory3;
              keepOutOfStore = true;
            };

            # Test derivation directory is symlinked as a single entry (no recursion)
            "derivation-dir".source = pkgs.runCommand "test-derivation-dir" { } ''
              mkdir -p $out
              echo "hello" > $out/hello.txt
            '';

            vars.text = ''
              . ${config.home.loadSessionVariables}
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

          # Test file with 3 variants works (file, text, directory)
          "foobar/foobar_2.txt" = {
            variants = {
              dark = {
                source = ./files/file_1;
                default = true;
              };

              light.text = ''
                Content
              '';

              other = {
                source = ./files/directory1;
              };
            };
          };

          # Test file with generator works
          "foobar/foobar_3.txt".text = lib.generators.toINI { } {
            main = {
              foo = "bar";
            };
          };
        };

        activationHooks = [
          ''echo "Hello World!"''
          "bat cache --build"
        ];
      }
    )
  ];
}
