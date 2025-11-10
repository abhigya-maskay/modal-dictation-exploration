# Agent Instructions

1. Always enter the Nix dev shell (`nix develop`) before running any Poetry, lint, format, or test commands. Use `nix develop --command <cmd>` when invoking single commands from tooling.
2. Prefer repo tools (`apply_patch`, built-in editors) over ad-hoc shell scripting for file edits.
3. Adhere to the project's existing instructions regarding Waybar indicator work and future front-end-first development flow.
