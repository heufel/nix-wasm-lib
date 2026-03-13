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
      path = "${out}/lib/nix_json.wasm";
    };

  testJson5 = ''
    {
      a: 0x1,
      b: "b",
      c: ["d", +2.5,],
      "e": {f: -3},
    }
  '';
in
let
  fromJson5 = wasm "fromJSON5" testJson5;
  toJson5 = wasm "toJSON5" fromJson5;
  fromJson5' = wasm "fromJSON5" toJson5;
in
assert (fromJson5 == testNix);
assert (fromJson5' == testNix);
drv (wasm "toJSON5" fromJson5')
