{
  buildEnv,
  callPackage,
  extraOutputsToInstall ? [ ],
  files ? [ ],
  activationScript ? null,
  lib,
  nix,
  packages ? [ ],
  pathsToLink ? [ ],
  pkgs,
  user,
}:
let
  inherit (lib) getExe;

  inherit (pkgs)
    callPackage
    writeShellScriptBin
    ;

  manifest = callPackage ./manifest.nix {
    inherit
      files
      user
      ;
  };

  linker = callPackage ../../heim/package.nix { doCheck = false; };

  activate = writeShellScriptBin "heim-activate" ''
    ${getExe linker} activate ${manifest} "$@"

    ${lib.optionalString (activationScript != null) activationScript}
  '';

  deactivate = writeShellScriptBin "heim-deactivate" ''
    ${getExe linker} deactivate ${manifest} "$@"
  '';

  nixCommand = "${getExe nix} --extra-experimental-features \"nix-command\"";

  switch = writeShellScriptBin "heim-switch" ''
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

    "$TARGET/nix/profile/bin/heim-activate"
  '';

  # Passthru script to aid in initial installation
  install = writeShellScriptBin "install" ''
    ${getExe switch} ${environment}
  '';

  environment = buildEnv {
    name = "heim-environment";

    paths = packages ++ [
      activate
      deactivate
      linker
      switch
    ];

    inherit
      pathsToLink
      extraOutputsToInstall
      ;

    passthru = {
      inherit
        activate
        deactivate
        environment
        install
        linker
        manifest
        ;
    };
  };
in
environment
