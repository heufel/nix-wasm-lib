# nix-wasm-lib
Crate around [DeterminateSystems](https://github.com/DeterminateSystems/)' `builtins.wasm` feature and their [nix_wasm_rust](https://github.com/DeterminateSystems/nix-wasm-rust) crate.

Provides serialization between `nix` and the `JSON`, `JSON5` and `TOML` formats (both ways).
Also provides the `nix_types` and `export_nix` crates, which make working with nix a lot easier (`nix_types::NixValue` should be similar to using Serde types like serde_json::Value).

#### `export_nix` warning
I wanted to try writing the `export_nix` macro without using any external crates, so it isn't very robust.
I still have to handle doc comments, and it seems to generate errors for types with `(...)` inside (or at least, it generated an error for `A<B<()>>`, but not `A<B<C>>`).
I also haven't given it good error reporting yet...
