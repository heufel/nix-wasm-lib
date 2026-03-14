{
  testNix,
  drv,
  wasm-fns,
  ...
}:
let
  wasm = wasm-fns { evaluator = builtins.wasm; };

  testJson = ''
    {
      "a": 1,
      "b": "b",
      "c": ["d", 2.5],
      "e": {"f": -3}
    }
  '';
in
let
  fromJson = wasm.fromJSON testJson;
  toJson = wasm.toJSON fromJson;
  fromJson' = wasm.fromJSON toJson;
in
assert (fromJson == testNix);
assert (fromJson' == testNix);
drv (wasm.toJSON fromJson')
