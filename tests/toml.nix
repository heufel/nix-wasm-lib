{
  out,
  testNix,
  drv,
  ...
}:
let
  wasm =
    function:
    builtins.wasm {
      inherit function;
      path = "${out}/lib/nix_toml.wasm";
    };

  testToml = ''
    a = 1
    b = "b"
    c = ["d", 2.5]
    [e]
    f = 3
  '';
in
let
  fromToml = wasm "fromTOML" testToml;
  toToml = wasm "toTOML" fromToml;
  fromToml' = wasm "fromTOML" toToml;
in
assert (fromToml == testNix);
assert (fromToml' == testNix);
drv (wasm "toTOML" fromToml')
