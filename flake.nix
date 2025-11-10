{
  description = "Modal Dictation Exploration development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        python = pkgs.python312;
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            python
            poetry
            black
            ruff
            pyright
            pre-commit
          ];

          shellHook = ''
            export POETRY_VIRTUALENVS_IN_PROJECT=true
            export PYTHONUTF8=1
            if [ ! -f poetry.lock ] && [ -f pyproject.toml ]; then
              echo "Run 'poetry install' to create the poetry.lock file."
            fi
          '';
        };
      });
}
