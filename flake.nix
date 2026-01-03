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
        # Build portaudio from git master with PulseAudio support
        # PulseAudio backend was added in Sept 2023, after v19.7.0 release
        portaudio-pulse = pkgs.stdenv.mkDerivation {
          pname = "portaudio";
          version = "unstable-2024-12-27";

          src = pkgs.fetchFromGitHub {
            owner = "PortAudio";
            repo = "portaudio";
            rev = "be9e16029ae9c33ef156fdd6d186996017dc8bdd";
            sha256 = "sha256-SaI2iUo5ZuIFKIyxdGiP5XIW/QVFeX0QjP5/2iPrK4M=";
          };

          nativeBuildInputs = [ pkgs.cmake pkgs.pkg-config ];
          buildInputs = [ pkgs.alsa-lib pkgs.libjack2 pkgs.libpulseaudio ];

          cmakeFlags = [
            "-DCMAKE_BUILD_TYPE=Release"
            "-DPA_USE_ALSA=ON"
            "-DPA_USE_JACK=ON"
            "-DPA_USE_PULSEAUDIO=ON"
            "-DCMAKE_INSTALL_LIBDIR=lib"
            "-DCMAKE_INSTALL_INCLUDEDIR=include"
          ];

          meta = {
            description =
              "Portable cross-platform Audio API (with PulseAudio support)";
            homepage = "https://www.portaudio.com/";
            license = pkgs.lib.licenses.mit;
          };
        };
      in {
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
            stdenv.cc.cc.lib # libstdc++

            # GTK/GObject for pystray menu support
            cairo
            cairo.dev
            glib
            glib.dev
            gobject-introspection
            gtk3

            # Audio support
            libpulseaudio
            portaudio-pulse

            # Development utilities
            git
            curl
            cacert
          ];

          shellHook = ''
            # Ensure SSL certificates are available
            export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt

            # GObject introspection for PyGObject
            export GI_TYPELIB_PATH="${pkgs.gtk3}/lib/girepository-1.0:${pkgs.glib}/lib/girepository-1.0:${pkgs.gobject-introspection}/lib/girepository-1.0''${GI_TYPELIB_PATH:+:$GI_TYPELIB_PATH}"

            # Library path for native libraries (libpulse for pulsectl, portaudio for sounddevice)
            export LD_LIBRARY_PATH="${portaudio-pulse}/lib:${pkgs.libpulseaudio}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

            # Use GTK backend for pystray (required for menu support on X11/bspwm)
            export PYSTRAY_BACKEND=gtk
          '';
        };
      });
}
