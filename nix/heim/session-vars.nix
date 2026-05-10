{
  lib,
  sessionVariables ? { },
  writeTextFile,
}:
let
  inherit (lib)
    concatStringsSep
    mapAttrsToList
    ;

  export = name: value: ''export ${name}="${toString value}"'';
in
writeTextFile {
  name = "heim-session-vars";
  destination = "/share/heim/session-vars.sh";
  text = concatStringsSep "\n" (mapAttrsToList export sessionVariables);
}
