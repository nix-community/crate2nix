{ inputs, lib, ... }: {
  imports = [
    inputs.devshell.flakeModule
  ];

  config.perSystem =
    { pkgs
    , ...
    }: {
      config.devshells.default = {
        imports = [
          "${inputs.devshell}/extra/language/c.nix"
          # "${inputs.devshell}/extra/language/rust.nix"
        ];

        commands = with pkgs; [
          { package = rust-toolchain; category = "rust"; }
        ];

        language.c = {
          libraries = lib.optional pkgs.stdenv.isDarwin pkgs.libiconv;
        };
      };
    };
}
