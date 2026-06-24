{
  __functor =
    self: pkgs:
    {
      modules ? [ ],
      specialArgs ? { },
    }:
    (pkgs.callPackage ./nix/heim { inherit modules specialArgs; });

  nixosModules.default = ./nix/nixos;
}
