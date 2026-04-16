{
  buildEnv,
  callPackage,
  lib,
  modules ? [ ],
  nix,
  pkgs,
  specialArgs ? { },
  writeShellScriptBin,
  writeText,
}:
let
  inherit (lib) getExe;

  evaluated = lib.evalModules {
    class = "heim";

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

  manifest = callPackage ./manifest.nix { inherit (cfg) files; };

  linker = callPackage ../../heim/package.nix { };

  nixCommand = "${getExe nix} --extra-experimental-features \"nix-command\"";

  activationScript = writeShellScriptBin "heim-activate" ''
    ${getExe linker} activate ${manifest}
  '';

  deactivationScript = writeShellScriptBin "heim-deactivate" ''
    ${getExe linker} deactivate ${manifest}
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

    ${getExe activationScript}
  '';

  installScript = writeShellScriptBin "install" ''
    ${getExe switchScript} ${profile}
  '';

  profile = buildEnv {
    name = "heim-environment";

    paths = cfg.packages ++ [
      linker
      activationScript
      deactivationScript
      switchScript
    ];

    inherit (cfg)
      pathsToLink
      extraOutputsToInstall
      ;

    passthru = {
      inherit manifest;
      activate = activationScript;
      deactivate = deactivationScript;
      install = installScript;
    };
  };
in
profile
