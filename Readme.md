# WebAssembly String Example
This is an experimental working example, to work with strings in between Module-Host and Guest-Module.

## Using
Run module-host with cargo run or modify guest-module and build via Makefile make, then copy compiled wasm module in module-host/src

## Multi-Value Return
Currently, Functions can only return a single I32, I64, F32 or F64 and MVR did not work for me yet - see comments in lib.rs and main.rs for more info