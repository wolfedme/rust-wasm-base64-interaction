use std::cell::Cell;
use wasmer::{Universal, Array, Instance, MemoryView, Module, Store, Value};
use wasmer_wasi::WasiState;

use base64_helper::{decode_with_termination, encode_with_termination};

fn b64_test(){
    let arr = [1, 2, 3];
    let en = encode_with_termination(&arr).unwrap();
    let de = decode_with_termination(&en).unwrap();

    println!("original: {:?}", arr);
    println!("encoded: {:?}", en);
    println!("decoded: {:?}", de);

}


fn main() {
    b64_test();
    // Get module bytes
    let wasm_bytes = std::fs::read("./src/guest_module_singleval.wasm").unwrap();
    // Use of Cranelift or LLVM to support Multi-Value-Returns!
    // Cranelift compiles faster, LLVM runs faster but compiles slower
    // Wasmer recommends LLVM
    // The LLVM compiler requires a valid installation of LLVM in your system. It currently requires LLVM 12.
    // ´sudo apt install llvm´ & ´cargo clean´ afterwards
    // Ubuntu needed additional libs: ´apt install gcc-multilib zlib1g-dev pkg-config libssl-dev libclang-common-13-dev libpq-dev´


    // Create LLVM compiler object
    let compiler = wasmer_compiler_llvm::LLVM::default();
    let store = Store::new(&Universal::new(compiler).engine());
    let module = Module::new(&store, wasm_bytes).unwrap();
    let mut wasi_env = WasiState::new("string_test").finalize().unwrap();
    let import_object = wasi_env.import_object(&module).unwrap();
    let instance = Instance::new(&module, &import_object).unwrap();

    let greet = instance.exports.get_function("greet").unwrap();
    let allocate = instance.exports.get_function("allocate").unwrap();
    let deallocate = instance.exports.get_function("deallocate").unwrap();

    // subject to bytes
    let subject = "Wasmer";
    // length to allocate
    let subject_length = subject.len();

    // get pointer as boxed wasmer value & allocate
    let input_pointer = allocate.call(&[Value::I32(subject_length as i32)]).unwrap();
    // unboxed wasmer value
    let input_pointer_unpacked = &input_pointer[0];
    // convert back to native usize
    let input_pointer_native = Value::unwrap_i32(input_pointer_unpacked) as usize;

    println!("pointer: {:?}", input_pointer);
    println!("pointer unpacked: {:?}", input_pointer_unpacked);
    println!("pointer native: {:?}", input_pointer_native);

    // https://radu-matei.com/blog/practical-guide-to-wasm-memory/
    let memory = instance.exports.get_memory("memory").unwrap();
    // https://github.com/wasmerio/wasmer-rust-example/blob/master/examples/string.rs
    // write into linear memory
    for (byte, cell) in subject
        .bytes()
        .zip(memory.view()[input_pointer_native as usize..input_pointer_native + subject_length].iter())
    {
        cell.set(byte);
    }

    // read mem
    let view: MemoryView<u8> = memory.view();
    for byte in view[input_pointer_native as usize..input_pointer_native + subject_length].iter().map(Cell::get) {
        println!("byte: {}", byte);
    }

    let in_vec: Vec<u8> = memory.view()
        [input_pointer_native as usize..(input_pointer_native + subject_length)]
        .iter()
        .map(|cell| cell.get())
        .collect();

    println!("str_vec: {:?}", in_vec);
    // Convert the subslice to a `&str`.
    let string = std::str::from_utf8(&in_vec).unwrap();

    println!("inputstring from mem: {:?}", string);

    let output = greet
        .call(&[Value::I32(input_pointer_native as i32)])
        .unwrap();

    println!("Output: {:?}", output);

    let output_pointer_native = Value::unwrap_i32(&output[0]);

    // TODO! Currently reads from pointer to memory end. Need to supply additional len
    let out_vec: Vec<u8> = memory.view()
    [output_pointer_native as usize..(output_pointer_native + 14) as usize]
    .iter()
    .map(|cell| cell.get())
    .collect();

    println!("out: {:?}", out_vec);
    println!("out: {:?}", std::str::from_utf8(&out_vec).unwrap());

    // TODO: Deallocate mem
}
