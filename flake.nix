{
  description = "coderunner with Docker image and NixOS service";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs { inherit system; };

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
            ] ++ compilerPackages;
          };

          dockerImage = pkgs.dockerTools.buildImage {
            name = "coderunner";
            tag = "latest";
            created = "now";
            contents = [ 
              coderunner 
              pkgs.bash 
              pkgs.coreutils
              pkgs.findutils
              pkgs.gnused
              pkgs.gnugrep
            ] ++ compilerPackages;
            config = {
              Env = [ 
                "PATH=${coderunner}/bin:${pkgs.lib.makeBinPath (compilerPackages ++ [ pkgs.bash pkgs.coreutils pkgs.findutils pkgs.gnused pkgs.gnugrep ])}"
              ];
              Cmd = [ "${coderunner}/bin/comphub" ];
              ExposedPorts = {
                "5000/tcp" = {};
              };
            };
          };
        in
        {
          packages = {
            default = coderunner;
            docker = dockerImage;
          };
        }
      ) // {
      nixosModules.coderunner = { config, pkgs, lib, ... }:
        with lib;
        let
          cfg = config.services.coderunner;
          dockerImage = self.packages.${pkgs.system}.docker;
        in
        {
          options.services.coderunner = {
            enable = mkEnableOption "coderunner backend";
            
            imageName = mkOption {
              type = types.str;
              default = "coderunner:latest";
              description = "Name and tag of the coderunner Docker image";
            };
            
            containerName = mkOption {
              type = types.str;
              default = "coderunner";
              description = "Name of the Docker container";
            };
            
            port = mkOption {
              type = types.port;
              default = 8080;
              description = "Port to expose the coderunner service on";
            };
            
            hostPort = mkOption {
              type = types.port;
              default = 8080;
              description = "Host port to bind to";
            };
            
            extraDockerArgs = mkOption {
              type = types.listOf types.str;
              default = [];
              description = "Additional arguments to pass to docker run";
            };
          };

          config = mkIf cfg.enable {
            virtualisation.docker.enable = true;

            systemd.services.coderunner = {
              description = "Coderunner Docker Service";
              after = [ "docker.service" ];
              requires = [ "docker.service" ];
              wantedBy = [ "multi-user.target" ];

              serviceConfig = {
                Type = "forking";
                RemainAfterExit = true;
                
                ExecStartPre = [
                  "${pkgs.docker}/bin/docker load -i ${dockerImage}"
                  "-${pkgs.docker}/bin/docker rm -f ${cfg.containerName}"
                  "-${pkgs.docker}/bin/docker stop ${cfg.containerName}"
                ];
                
                ExecStart = ''
                  ${pkgs.docker}/bin/docker run \
                    --detach \
                    --name ${cfg.containerName} \
                    --publish ${toString cfg.hostPort}:${toString cfg.port} \
                    --restart unless-stopped \
                    ${lib.concatStringsSep " " cfg.extraDockerArgs} \
                    ${cfg.imageName}
                '';
                
                ExecStop = "${pkgs.docker}/bin/docker stop ${cfg.containerName}";
                ExecStopPost = "${pkgs.docker}/bin/docker rm -f ${cfg.containerName}";
                
                Restart = "on-failure";
                RestartSec = "5s";
                TimeoutStartSec = "120s";
                TimeoutStopSec = "30s";
              };
              
              unitConfig = {
                StartLimitBurst = 3;
                StartLimitInterval = "60s";
              };
            };
          };
        };
    };
}
