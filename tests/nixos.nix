let
  sources = import ../npins;
  pkgs = import sources.nixpkgs {
    config = { };
    overlays = [ ];
  };
in
pkgs.nixos [
  (
    { pkgs, ... }:
    {
      imports = [
        (import ../.).nixosModules.default
      ];

      boot.loader.systemd-boot.enable = true;

      virtualisation.vmVariant = {
        virtualisation.graphics = false;
      };

      users.users.nixos = {
        isNormalUser = true;

        heim = {
          overwrite = true;

          home = {
            files = {
              "directory1" = {
                source = ./files/directory1;
                overwrite = false;
              };
              "directory2".source = ./files/directory2;
            };
          };

          xdg.config.files = {
            "foobar/foobar_1.txt".source = ./files/file_1;
            "foobar/foobar_2.txt".text = "foobar";
          };
        };

        packages = [ pkgs.htop ];
      };
    }
  )
]
