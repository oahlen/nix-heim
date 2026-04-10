{
  __functor =
    self: pkgs: modules:
    pkgs.callPackage ./nix { inherit modules; };
}
