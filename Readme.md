# Working with Arrays between WebAssembly & Rust
Experimental transference of arrays via ´BASE64´ encoded strings to enable working with Arrays and other more complex data types, until the Multi-Value- and Interface-Type Proposals leave experimental state.

´BASE64´ strings get written into the modules memory and functions only return pointers to these strings, which contain a termination sequence to indicate, when the string ends.

## Using
Run module-host with cargo run or modify guest-module and build via Makefile make, then copy compiled wasm module in module-host/src