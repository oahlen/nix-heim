{
  buildEnv,
  callPackage,
  extraOutputsToInstall ? [ ],
  files ? [ ],
  lib,
  nix,
  packages ? [ ],
  pathsToLink ? [ ],
  pkgs,
}:
let
  inherit (lib) getExe;

  inherit (pkgs)
    callPackage
    writeShellScriptBin
    ;

  manifest = callPackage ./manifest.nix { inherit files; };

  linker = callPackage ../../heim/package.nix { doCheck = false; };

  activate = writeShellScriptBin "heim-activate" ''
    ${getExe linker} activate ${manifest} "$@"
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

    ${getExe activate}
  '';

  install = writeShellScriptBin "install" ''
    ${getExe switch} ${environment}
  '';

  environment = buildEnv {
    name = "heim-environment";

    paths = packages ++ [
      linker
      activate
      deactivate
      switch
    ];

    inherit
      pathsToLink
      extraOutputsToInstall
      ;

    passthru = {
      inherit
        manifest
        linker
        activate
        deactivate
        install
        environment
        ;
    };
  };
in
environment
