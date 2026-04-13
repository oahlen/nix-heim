{
  __functor =
    self: pkgs: modules:
    pkgs.callPackage ./nix { inherit modules; };

  nixosModules.default = ./nix/nixos;
}
