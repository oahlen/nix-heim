{
  __functor =
    self: pkgs: modules:
    (pkgs.callPackage ./nix/heim { inherit modules; });

  nixosModules.default = ./nix/nixos;
}
