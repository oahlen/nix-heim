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

  destination = "/share/heim/session-vars.sh";

  export = name: value: ''export ${name}="${toString value}"'';

  prependToPath = v: "${concatStringsSep ":" v}\${PATH:+:}\$PATH";
in
{
  path = destination;

  package = writeTextFile {
    name = "heim-session-vars";
    destination = destination;
    text = ''
      ${concatStringsSep "\n" (mapAttrsToList export sessionVariables)};

      ${export "PATH" (prependToPath sessionPath)}
    '';
  };
}
