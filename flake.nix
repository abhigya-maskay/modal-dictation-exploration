{
  description = "Phonesc - Modal voice dictation system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer

            # Build tools
            pkg-config
            cmake
            ninja
            clang
            gcc

            # Audio dependencies
            pipewire
            pulseaudio
            alsa-lib
            jack2

            # Wayland dependencies
            wayland
            wayland-protocols
            libxkbcommon
            xorg.libxcb
            xorg.libX11

            # ML/ASR dependencies
            onnxruntime

            # Additional libraries
            openssl
            sqlite
          ];

          # Environment variables for proper Rust builds
          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.lib.makeLibraryPath [
              pkgs.alsa-lib
              pkgs.pipewire
              pkgs.pulseaudio
              pkgs.jack2
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.onnxruntime
              pkgs.openssl
            ]}/pkgconfig:$PKG_CONFIG_PATH"

            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
              pkgs.alsa-lib
              pkgs.pipewire
              pkgs.pulseaudio
              pkgs.jack2
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.onnxruntime
              pkgs.openssl
            ]}:$LD_LIBRARY_PATH"

            echo "🚀 Phonesc development environment loaded"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
          '';
        };
      }
    );
}
