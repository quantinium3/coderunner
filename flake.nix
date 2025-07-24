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

          bin = pkgs.rustPlatform.buildRustPackage rec {
            pname = "coderunner";
            version = "0.1.0";
            src = pkgs.fetchFromGitHub {
              owner = "quantinium3";
              repo = "coderunner";
              rev = "c3c53a1ee8242a17a8cd9625b5b08a3704aca63b";
              sha256 = "sha256-BlCmm8a36FYEUClyqPRv2N8n6/DpiKgI7ivwFSyVaPU=";
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
          packages = {
            default = bin;
          };
        }
      ) // {
      nixosModules.coderunner = { config, pkgs, lib, ... }:
        {
          options.services.coderunner = {
            enable = lib.mkEnableOption "coderunner backend";
          };
/*           config = lib.mkIf config.services.coderunner.enable { */
        };
    };
}
