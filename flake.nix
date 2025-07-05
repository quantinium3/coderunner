{
  description = "A development environment with OpenSSL for Rust projects";

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
            pkgs.openssl
            pkgs.pkg-config
          ];

          env = {
            OPENSSL_DIR = "${openssl.dev}";
            OPENSSL_LIB_DIR = "${openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
            
            PKG_CONFIG_PATH = "${openssl.out}/lib/pkgconfig";
            
            LD_LIBRARY_PATH = "${openssl.out}/lib";
          };

          shellHook = ''
            export RUSTFLAGS="-C link-arg=-L${openssl.out}/lib"
            echo "OpenSSL environment configured:"
            echo "  OPENSSL_DIR: $OPENSSL_DIR"
            echo "  OPENSSL_LIB_DIR: $OPENSSL_LIB_DIR"
            echo "  PKG_CONFIG_PATH: $PKG_CONFIG_PATH"
          '';
        };
      }
    );
}
