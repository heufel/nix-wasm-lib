build:
	nix build

WASM_INS=ins
WASM_FUNCTION="{                           \
  $(WASM_INS) = {                          \
    toml =                                 \
      function:                            \
      builtins.wasm {                      \
        inherit function;                  \
        path = ./result/lib/nix_toml.wasm; \
      };                                   \
    json =                                 \
      function:                            \
      builtins.wasm {                      \
        inherit function;                  \
        path = ./result/lib/nix_json.wasm; \
      };                                   \
  };                                       \
}"
.PHONY: tmp
.SILENT: tmp
tmp: 
	echo $(WASM_FUNCTION) > tmp.nix

.PHONY: repl
.SILENT: repl
repl: tmp
	dnix --extra-experimental-features wasm-builtin repl --file tmp.nix $(WASM_INS) || true
	rm tmp.nix -f

.PHONY: test
test:
	dnix --extra-experimental-features wasm-builtin flake check

.PHONY: test-v
test-v:
	dnix --extra-experimental-features wasm-builtin flake check --print-build-logs
