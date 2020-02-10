# Throws an error if any of our lib tests fail.
let
  pkgs = import ../../../../../nixpkgs.nix {};
  crate2nix = pkgs.callPackage ../default.nix {};
  tests = [ "dependencyDerivations" "dependencyFeatures" "expandFeatures" "packageFeatures" ];
  runTest = f: {
    "00testName" = f;
    failures = (pkgs.callPackage (./. + "/${f}.nix")) { inherit crate2nix; };
  };
  all = builtins.filter (r: r.failures != []) (builtins.map runTest tests);
in
if all == []
then "OK"
else all
