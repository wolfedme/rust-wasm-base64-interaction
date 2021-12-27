.DEFAULT_GOAL := build_module_default

build_module_default:
	cargo build --target wasm32-wasi

build_module_multiValue:
	cargo rustc -Zmultitarget --target=wasm32-wasi -- -C target-feature=+multivalue