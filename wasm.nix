{
  craneLib,
  pkgs,
  lib ? pkgs.lib,
  target,
  ...
}:
let
  inherit (builtins)
    readFile
    readDir
    attrNames
    attrValues
    listToAttrs
    filter
    head
    replaceStrings
    mapAttrs
    elem
    ;
  inherit (lib) filterAttrs optional mergeAttrsList;
  wasm-ld =
    let
      version = "0.5.21";
      name = "wasm-component-ld";
      src = fetchGit {
        url = "https://github.com/bytecodealliance/${name}.git";
        ref = "refs/tags/v${version}";
        rev = "0a5f2513f6157e2c01c0485aa68365d8e4622ccb";
      };
    in
    pkgs.rustPlatform.buildRustPackage {
      inherit name version src;
      cargoLock.lockFile = "${src}/Cargo.lock";
      doCheck = false;
    };

  nix-wasm-crate =
    name: version:
    craneLib.buildPackage {
      inherit name version;
      buildInputs = [ wasm-ld ];
      src = ./.;
      cargoToml = ./Cargo.toml;
      cargoVendorDir = craneLib.vendorCargoDeps {
        cargoLock = ./Cargo.lock;
      };
      strictDeps = true;

      doCheck = false;
      cargoExtraArgs = "--target ${target}";
    };

  exports =
    let
      version = (fromTOML (readFile ./Cargo.toml)).workspace.package.version;

      dirs = map (dir: ./crates/${dir}) (
        attrNames (filterAttrs (_: t: t == "directory") (readDir ./crates))
      );

      is-crate = dir: elem "Cargo.toml" (attrNames (readDir dir));
      read-crate = crate: fromTOML (readFile "${crate}/Cargo.toml");
      crates = map read-crate (filter is-crate dirs);

      get-exports =
        toml: optional (toml ? package.metadata.nix-exports) toml.package.metadata.nix-exports;

      output-set = map (crate: rec {
        name = crate.package.name;
        value = {
          package = nix-wasm-crate name version;
          functions = get-exports crate;
        };
      }) crates;
      output-filtered = filter (crate: crate.value.functions != [ ]) output-set;
    in
    listToAttrs output-filtered;

in
rec {
  inherit wasm-ld;
  wasm =
    let
      separated = mapAttrs (_: prev: prev.package) exports;
    in
    separated
    // {
      wasm = pkgs.buildEnv {
        name = "nix-wasm";
        paths = attrValues separated;
      };
    };
  functions =
    {
      package ? wasm.wasm,
      evaluator,
    }:
    let
      separated = mapAttrs (
        name: prev:
        listToAttrs (
          map (function: {
            name = function;
            value = evaluator {
              function = function;
              path = "${package}/lib/${replaceStrings [ "-" ] [ "_" ] name}.wasm";
            };
          }) (head prev.functions) # no clue why head has to be called here...
        )
      ) exports;
    in
    separated // mergeAttrsList (attrValues separated);
}
