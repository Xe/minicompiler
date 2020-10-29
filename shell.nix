{ sources ? import ./nix/sources.nix, pkgs ? import sources.nixpkgs { }
, rust ? pkgs.callPackage ./nix/rust.nix { } }:

pkgs.mkShell { buildInputs = with pkgs; [ rust ]; }
