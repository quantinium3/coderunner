FROM nixos/nix:latest AS builder
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
RUN nix-channel --update
COPY . /tmp/build
WORKDIR /tmp/build

# Build with flakes
RUN nix build

# Copy the closure properly
RUN nix-store --export $(nix-store -qR result/) > /tmp/closure.nar

FROM nixos/nix
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
RUN nix-channel --update
WORKDIR /app

RUN nix-env -iA nixpkgs.bun nixpkgs.zig nixpkgs.crystal nixpkgs.dmd nixpkgs.dart nixpkgs.go nixpkgs.groovy nixpkgs.ghc nixpkgs.julia nixpkgs.nix nixpkgs.odin nixpkgs.perl nixpkgs.ruby nixpkgs.rustc nixpkgs.scala nixpkgs.bfc nixpkgs.R nixpkgs.clang nixpkgs.python3 nixpkgs.luaPackages.lua

# Import the closure properly
COPY --from=builder /tmp/closure.nar /tmp/
RUN nix-store --import < /tmp/closure.nar

COPY --from=builder /tmp/build/result /app
CMD ["/app/bin/comphub"]
