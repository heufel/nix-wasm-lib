{
  craneLib,
  pkgs,
  lib ? pkgs.lib,
  target,
  ...
}:
let
  wasm-ld =
    let
      version = "0.5.21";
      name = "wasm-component-ld";
      src = fetchGit {
        url = "https://github.com/bytecodealliance/wasm-component-ld.git";
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
      buildInputs = [ wasm-ld ] ++ (with pkgs; lib.optionals stdenv.isDarwin [ libiconv ]);
      src = ./.;
      cargoToml = ./Cargo.toml;
      cargoVendorDir = craneLib.vendorCargoDeps {
        cargoLock = ./Cargo.lock;
      };
      strictDeps = true;

      doCheck = false;
      cargoExtraArgs = "--target ${target}";
    };
  read-cargo-toml = crate: fromTOML (builtins.readFile "${crate}/Cargo.toml");

  wasm =
    with builtins;
    with lib;
    let
      dirs = map (dir: ./crates/${dir}) (
        attrNames (filterAttrs (n: t: t == "directory") (readDir ./crates))
      );
      is-crate = dir: elem "Cargo.toml" (attrNames (readDir dir));
      is-cdylib =
        crate:
        let
          toml = read-cargo-toml crate;
        in
        (toml ? lib.crate-type) && elem "cdylib" toml.lib.crate-type;
      cdylibs = filter is-cdylib (filter is-crate dirs);
      get-name-version =
        crate:
        let
          toml = read-cargo-toml crate;
        in
        {
          name = toml.package.name;
          version = toml.package.version;
        };
    in
    listToAttrs (
      map (
        lib:
        let
          nv = get-name-version lib;
        in
        {
          name = nv.name;
          value = nix-wasm-crate nv.name nv.version;
        }
      ) cdylibs
    );

in
{
  inherit wasm-ld wasm;
}
