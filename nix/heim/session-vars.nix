{
  lib,
  sessionPath ? [ ],
  sessionVariables ? { },
  writeTextFile,
}:
let
  inherit (lib)
    concatStringsSep
    mapAttrsToList
    ;

  export = name: value: ''export ${name}="${toString value}"'';

  prependToPath = v: "${concatStringsSep ":" v}\${PATH:+:}\$PATH";
in
writeTextFile {
  name = "heim-session-vars";
  destination = "/share/heim/session-vars.sh";
  text = ''
    ${concatStringsSep "\n" (mapAttrsToList export sessionVariables)};

    ${export "PATH" (prependToPath sessionPath)}
  '';
}
