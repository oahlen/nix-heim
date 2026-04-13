{
  buildEnv,
  lib,
  modules ? [ ],
  pkgs,
  specialArgs ? { },
  writeShellScriptBin,
  writeText,
}:
let
  inherit (import ./manifest.nix { inherit lib pkgs; })
    generateManifest
    ;

  evaluated = lib.evalModules {
    specialArgs = {
      inherit pkgs lib;
    }
    // specialArgs;
    modules = [
      ./modules/user.nix
    ]
    ++ modules;
  };

  cfg = evaluated.config;

  files = [
    cfg.home.files
    cfg.xdg.config.files
    cfg.xdg.data.files
    cfg.xdg.state.files
  ];

  manifest = writeText "manifest.json" (generateManifest files);

  linker = pkgs.callPackage ../heim { };

  nixCommand = "${lib.getExe pkgs.nix} --extra-experimental-features \"nix-command\"";

  activationScript = writeShellScriptBin "heim-activate" ''
    ${lib.getExe linker} activate ${manifest}
  '';

  deactivationScript = writeShellScriptBin "heim-deactivate" ''
    ${lib.getExe linker} deactivate ${manifest}
  '';

  switchScript = writeShellScriptBin "heim-switch" ''
    TARGET=''${XDG_STATE_HOME:-$HOME/.local/state}
    mkdir -p "$TARGET/nix/profiles"

    FILE="$1"
    ATTR="$2"

    if [[ -z "$FILE" ]]; then
      echo "Error: no file provided"
      echo "Usage: heim-switch <file> [attribute]"
      exit 1
    fi

    if [[ "$FILE" == /nix/store/* ]]; then
      ${nixCommand} build "$FILE" --profile "$TARGET/nix/profiles/profile" --no-link
    else
      ${nixCommand} build -f "$FILE" $ATTR --profile "$TARGET/nix/profiles/profile" --no-link
    fi

    ln -sfn "$TARGET/nix/profiles/profile" "$TARGET/nix/profile"

    ${lib.getExe activationScript}
  '';

  profile = buildEnv {
    name = "heim-environment";
    paths = cfg.home.packages ++ [
      activationScript
      deactivationScript
      switchScript
    ];
    inherit (cfg.home)
      pathsToLink
      extraOutputsToInstall
      ;
  };
in
profile
// {
  inherit manifest;

  activate = activationScript;
  deactivate = deactivationScript;

  install = writeShellScriptBin "install" ''
    ${lib.getExe switchScript} ${profile}
  '';
}
