{
  description = "A flake for this submodule, providing a basic development shell.";

  inputs = {
    nixpkgs.url = "github:meta-introspector/nixpkgs?ref=feature/CRQ-016-nixify";
    flake-utils.url = "github:meta-introspector/flake-utils?ref=feature/CRQ-016-nixify";
  };

  outputs = { self, nixpkgs, flake-utils, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            bash
            git
            shellcheck # Add shellcheck for shell script linting
            # Add any other common tools needed for your submodules here
          ];

          shellHook = ''
            echo "Welcome to the development shell of this submodule!"
          '';
        };
      }
    );
}
