{
  description = "Voice control software for Linux and macOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };
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

            # Audio device support for pulsectl
            libpulseaudio

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

            # Library path for native libraries (libpulse for pulsectl)
            export LD_LIBRARY_PATH="${pkgs.libpulseaudio}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

            # Use GTK backend for pystray (required for menu support on X11/bspwm)
            export PYSTRAY_BACKEND=gtk
          '';
        };
      });
}
