{
  # provider perSystem
  transposition.tools = { };

  perSystem = { config, pkgs, lib, ... }: {
    options.tools = lib.mkOption {
      description = ''
        Library functions to generate the `Cargo.nix` build
        file automatically.
      '';

      type = lib.types.submoduleWith {
        modules = [
          {
            options.generatedCargoNix = lib.mkOption {
              description = ''
                Returns a derivation containing the generated `Cargo.nix` file
                which can be called with `pkgs.callPackage`.

                name: will be part of the derivation name
                src: the source that is needed to build the crate, usually the
                crate/workspace root directory
                cargoToml: Path to the Cargo.toml file relative to src, "Cargo.toml" by
                default.
              '';
              type = lib.types.functionTo lib.types.package;
            };
            options.appliedCargoNix = lib.mkOption {
              description = ''
                Applies the default arguments from pkgs to the generated `Cargo.nix` file.
                
                name: will be part of the derivation name
                src: the source that is needed to build the crate, usually the crate/workspace root directory
                cargoToml: Path to the Cargo.toml file relative to src, "Cargo.toml" by default.
              '';
              type = lib.types.functionTo lib.types.attrs;
            };
          }
        ];
      };
    };

    config.tools =
      let tools = pkgs.callPackage ../../tools.nix { };
      in {
        inherit (tools) generatedCargoNix appliedCargoNix;
      };

    config.checks = {
      toolsGeneratedCargoNix_crate2nix =
        let
          cargoNixBuilder = config.tools.generatedCargoNix { name = "crate2nix"; src = ../../crate2nix; };
          cargoNix = pkgs.callPackage cargoNixBuilder { };
        in
        cargoNix.rootCrate.build;

      toolsAppliedCargoNix_crate2nix =
        let cargoNix = config.tools.appliedCargoNix { name = "crate2nix"; src = ../../crate2nix; };
        in cargoNix.rootCrate.build;
    };
  };
}
