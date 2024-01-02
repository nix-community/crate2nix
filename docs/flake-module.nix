{
  perSystem = { config, self', inputs', pkgs, system, ... }: {
    packages.docs = pkgs.buildNpmPackage {
      pname = "docs";
      version = "0.1.0";

      src = pkgs.nix-gitignore.gitignoreSource [
        ".vscode"
        "README.md"
        ".gitignore"
        "nix"
        "flake.*"
      ]
        ./.;

      buildInputs = [
        pkgs.vips
      ];

      nativeBuildInputs = [
        pkgs.pkg-config
      ];

      installPhase = ''
        runHook preInstall
        cp -pr --reflink=auto dist $out/
        runHook postInstall
      '';

      npmDepsHash = "sha256-5PLfsxFmN20+/BMYWP9hK5Aw0qV9XiG/Rky8BlF80J0=";
    };
  };
}
