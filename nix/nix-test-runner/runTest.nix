{ testFile ? null
, tests ? import testFile
, lib
}:
with builtins;
let
  testNames = map (t: { passedTest = t; }) (attrNames tests);
  failed = map
    (t: {
      failedTest = t.name;
      expected = prettyVal t.expected;
      result = prettyVal t.result;
    })
    (lib.debug.runTests tests);
  failedTests = map (f: f.failedTest) failed;
  passed = filter (t: !lib.elem t.passedTest failedTests) testNames;
  result = { inherit passed failed; };
  prettyVal =
    let
      modify = v:
        let
          pr = f: {
            __pretty = f;
            val = v;
          };
        in
        if lib.isDerivation v then
          pr (drv: "<δ:${drv.name}:${concatStringsSep "," (attrNames drv)}>")
        else if [ ] == v then
          pr (lib.const "[]")
        else if lib.isList v then
          pr (l: "[ ${toString (map go l)} ]")
        else if lib.isAttrs v then
          pr
            (a:
              "{ ${
            concatStringsSep " "
            (lib.attrValues (lib.mapAttrs (n: v: "${n} = ${go v};") v))
          } }")
        else
          v;
      go = x: lib.generators.toPretty { allowPrettyValues = true; } (modify x);
    in
    go;
in
result
