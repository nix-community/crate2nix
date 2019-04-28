# Throws an error if any of our lib tests fail.
let pkgs = import ../../../../nixpkgs.nix {};
    crate2nix = pkgs.callPackage ../default.nix {};
    tests = [ "dependencyFeatures" "expandFeatures" ];
    runTest = f: (pkgs.callPackage (./. + "/${f}.nix")) { inherit crate2nix; };
    all = builtins.concatLists (map runTest tests);
in if all == []
   then "OK"
   else throw (builtins.toJSON all)
