{
  description = "coderunner";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        coderunner = pkgs.rustPlatform.buildRustPackage rec {
          pname = "coderunner";
          version = "0.1.0";
          src = pkgs.fetchFromGitHub {
            owner = "quantinium3";
            repo = "coderunner";
            rev = "61153a03bcfba107541b1014329eeb0f184b567e";
            sha256 = "sha256-iv8kUHUywUrlbucIRqoen1PcSNSK9G86crPMzqRdN9U=";
          };
          doCheck = false;
          cargoHash = "sha256-cJ1h6RrUhjT0sGgEa0B6OP1luTdFWaz+8LtVyVeDwSs=";
          nativeBuildInputs = with pkgs; [
            pkg-config
            zig
            crystal
            dmd
            dart
            go
            groovy
            ghc
            julia
            nix
            odin
            perl
            ruby
            rustc
            scala
            bfc
            R
            clang
            bun
            python3
            go
            luaPackages.lua
            nix
            perl
            R
            ruby
            rustc
            scala
          ];
          buildInputs = with pkgs; [
            openssl
            zig
            crystal
            dmd
            dart
            go
            groovy
            ghc
            julia
            nix
            odin
            perl
            ruby
            rustc
            scala
            bfc
            R
            clang
            bun
            python3
            go
            luaPackages.lua
            nix
            perl
            R
            ruby
            rustc
            scala
          ];
        };
      in
      {
        packages.default = coderunner;
        packages.coderunner = coderunner;
      }
    );
}
