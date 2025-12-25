{
  description = "Voice control software for Linux and macOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # uv manages Python and all Python tooling
            uv

            # Build essentials for native extensions
            gcc
            gnumake
            pkg-config
            cmake

            # Common native libraries required by Python packages
            openssl
            openssl.dev
            zlib
            zlib.dev
            libffi
            libffi.dev
            stdenv.cc.cc.lib  # libstdc++

            # Development utilities
            git
            curl
            cacert
          ];

          shellHook = ''
            # Ensure SSL certificates are available
            export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
          '';
        };
      }
    );
}
