# nix-wasm-lib
Crate around [DeterminateSystems](https://github.com/DeterminateSystems/)' `builtins.wasm` feature and their [nix_wasm_rust](https://github.com/DeterminateSystems/nix-wasm-rust) crate.

Provides serialization between nix and the JSON and TOML formats (both ways).
Also provides the nix_types and export_nix crates, which make working with nix a lot easier (nix_types::NixValue should be similar to using Serde types like serde_json::Value).

#### `export_nix` warning
I wanted to try writing the `export_nix` macro without using any external crates, so it isn't very robust.
I still have to handle doc comments, and it isn't tested against any types that are even remotely complicated.
I also haven't given it good error reporting yet...
