{
  description = "idk man";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        openssl = pkgs.openssl;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            openssl
            pkgs.clang
            pkgs.kotlin
            pkgs.scala_3
            pkgs.groovy
            pkgs.dart
            pkgs.ruby
            pkgs.lua5
            pkgs.julia
            pkgs.R
            pkgs.perl
            pkgs.crystal
            pkgs.nim
            pkgs.ghc
            pkgs.elmPackages.elm
            pkgs.vlang
            pkgs.coffeescript
            pkgs.dotnet-sdk
            pkgs.fsharp
            pkgs.ocaml
            pkgs.elixir
            pkgs.erlang
            pkgs.php
            pkgs.gfortran
            pkgs.gnat
            pkgs.fpc
            pkgs.racket
            pkgs.clojure
            pkgs.bash
            pkgs.powershell
            pkgs.nasm
            pkgs.gprolog
            pkgs.fish
            pkgs.agda
            pkgs.purescript
            pkgs.gleam
            pkgs.odin
            pkgs.gforth
            pkgs.gst
            pkgs.tcl
            pkgs.jq
            pkgs.algol68g
            pkgs.idris2
            pkgs.factor-lang
            pkgs.sbcl
            pkgs.mercury
            pkgs.janet
            pkgs.swift
            pkgs.dmd
            pkgs.zig
            pkgs.bacon
            pkgs.pkg-config
            pkgs.chicken
            pkgs.chez
            pkgs.fennel
            pkgs.clang
            pkgs.gnustep-libobjc
            pkgs.gnustep-base
          ];

          env = {
            OPENSSL_DIR = "${openssl.dev}";
            OPENSSL_LIB_DIR = "${openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";

            PKG_CONFIG_PATH = "${openssl.out}/lib/pkgconfig";

            LD_LIBRARY_PATH = "${openssl.out}/lib:${pkgs.swiftPackages.Dispatch}/lib";
          };

          stdenv = pkgs.swift.stdenv;

          shellHook = ''
            export RUSTFLAGS="-C link-arg=-L${openssl.out}/lib"
          '';
        };
      }
    );
}
