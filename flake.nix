{
  description = "coderunner";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
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
            ];
            buildInputs = with pkgs; [
              openssl
            ];
          };
        in
        {
          packages.default = coderunner;
          packages.coderunner = coderunner;
        }
      ) // {
      nixosModules.coderunner = { config, pkgs, lib, ... }: 
        let
          compilerPackages = with pkgs; [
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
            luaPackages.lua
          ];
          
          compilerPaths = pkgs.lib.makeBinPath compilerPackages;
        in
        {
          options.services.coderunner = {
            enable = lib.mkEnableOption "coderunner backend";
          };
          config = lib.mkIf config.services.coderunner.enable {
            environment.systemPackages = compilerPackages;
            
            systemd.services.coderunner = {
              description = "coderunner backend";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];
              serviceConfig = {
                ExecStart = "${self.packages.${pkgs.system}.coderunner}/bin/comphub";
                Restart = "always";
                RestartSec = "10s";
                StandardOutput = "journal";
                StandardError = "journal";
                User = "nixie";
                Environment = "PATH=${compilerPaths}:$PATH";
              };
            };
          };
        };
    };
}
