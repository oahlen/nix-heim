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

  activationScript = writeShellScriptBin "activate" ''
    ${lib.getExe linker} activate ${manifest}
  '';

  deactivationScript = writeShellScriptBin "deactivate" ''
    ${lib.getExe linker} deactivate ${manifest}
  '';

  profile = buildEnv {
    name = "heim-environment";
    paths = cfg.home.packages ++ [
      activationScript
      deactivationScript
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

  switch =
    let
      nixCommand = "${lib.getExe pkgs.nix} --extra-experimental-features \"nix-command\"";
    in
    writeShellScriptBin "switch" ''
      TARGET=''${XDG_STATE_HOME:-$HOME/.local/state}
      mkdir -p "$TARGET/nix/profiles"
      ${nixCommand} build ${profile} --profile "$TARGET/nix/profiles/profile"
      ln -sfn "$TARGET/nix/profiles/profile" "$TARGET/nix/profile"

      ${lib.getExe activationScript}
    '';
}
