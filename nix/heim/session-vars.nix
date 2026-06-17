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

  destination = "/share/heim/session-vars.sh";

  package = writeTextFile {
    name = "heim-session-vars";

    inherit destination;

    text = ''
      ${concatStringsSep "\n" (mapAttrsToList export sessionVariables)};

      ${export "PATH" (prependToPath sessionPath)}
    '';
  };
in
{
  path = "${package}${destination}";
  inherit package;
}
