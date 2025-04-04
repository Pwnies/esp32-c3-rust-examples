{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
    in
    {
      devShell."${system}" =
        with pkgs;
        mkShell {
          nativeBuildInputs = [
            # nix develop shells will by default include a bash in the $PATH,
            # however this bash will be a non-interactive bash. The deviates from
            # how nix-shell works. This fix was taken from:
            #    https://discourse.nixos.org/t/interactive-bash-with-nix-develop-flake/15486
            bashInteractive

            # Rust
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [ "riscv32imc-unknown-none-elf" ];
            })

            # Flashing tool
            espflash
          ];

          shellHook = ''
            # nix develop shells will by default overwrite the $SHELL variable with a
            # non-interactive version of bash. The deviates from how nix-shell works.
            # This fix was taken from:
            #    https://discourse.nixos.org/t/interactive-bash-with-nix-develop-flake/15486
            #
            # See also: nixpkgs#5131 nixpkgs#6091
            export SHELL=${bashInteractive}/bin/bash
          '';
        };
    };
}
