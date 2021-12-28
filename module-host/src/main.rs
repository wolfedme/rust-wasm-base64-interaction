use std::cell::Cell;
use wasmer::{Array, Instance, MemoryView, Module, Store, Universal, Value};
use wasmer_wasi::WasiState;

use base64_helper::{
    decode_with_termination, encode_with_termination, get_max_length, get_termination_sequence,
};
fn main() {
    // Get module bytes
    let wasm_bytes = std::fs::read("./src/guest_module_base64.wasm").unwrap();
    // Use of Cranelift or LLVM to support Multi-Value-Returns!
    // Cranelift compiles faster, LLVM runs faster but compiles slower
    // Wasmer recommends LLVM
    // The LLVM compiler requires a valid installation of LLVM in your system. It currently requires LLVM 12.
    // ´sudo apt install llvm´ & ´cargo clean´ afterwards
    // Ubuntu needed additional libs: ´apt install gcc-multilib zlib1g-dev pkg-config libssl-dev libclang-common-13-dev libpq-dev´

    // -------------
    // Wasmer Init
    // -------------

    // Create LLVM compiler object & wasmer objects
    let compiler = wasmer_compiler_llvm::LLVM::default();
    let store = Store::new(&Universal::new(compiler).engine());
    let module = Module::new(&store, wasm_bytes).unwrap();
    let mut wasi_env = WasiState::new("string_test").finalize().unwrap();
    let import_object = wasi_env.import_object(&module).unwrap();
    let instance = Instance::new(&module, &import_object).unwrap();

    let greet = instance.exports.get_function("greet").unwrap();
    let allocate = instance.exports.get_function("allocate").unwrap();
    let deallocate = instance.exports.get_function("deallocate").unwrap();

    // -------------
    // reverse array
    // -------------

    // encode array into base64 with termination sequence
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let arr_encoded = encode_with_termination(&arr).unwrap();
    println!("{:?} encoded to {:?}", arr, arr_encoded);

    // allocate in module memory and get pointer
    let arr_ptr = &allocate
        .call(&[Value::I32(arr_encoded.len() as i32)])
        .unwrap()[0];
    let arr_ptr = Value::unwrap_i32(&arr_ptr) as usize;

    // write bytes into module memory
    let memory = instance.exports.get_memory("memory").unwrap();
    for (byte, cell) in arr_encoded
        .bytes()
        .zip(memory.view()[arr_ptr as usize..arr_ptr + arr_encoded.len()].iter())
    {
        cell.set(byte);
    }
    println!(
        "{:?} bytes written, starting at 0x{:?}",
        arr_encoded.len(),
        arr_ptr
    );

    // print subject in module memory
    _debug_print_mem_bytes(&memory, arr_ptr, arr_encoded.len());

    // call module reverse function
    let arr_reverse_ptr_to_encoded = instance
        .exports
        .get_function("reverse_array")
        .unwrap()
        .call(&[Value::I32(arr_ptr as i32)])
        .unwrap();
    // convert back to native value
    let arr_reverse_ptr_to_encoded = Value::unwrap_i32(&arr_reverse_ptr_to_encoded[0]) as usize;

    let arr_reverse_encoded = read_until_sequence(memory, arr_reverse_ptr_to_encoded);
    println!(
        "Found encoded string at address 0x{:?} + {:?}: {:?}",
        arr_reverse_ptr_to_encoded,
        arr_reverse_encoded.len(),
        arr_reverse_encoded
    );

    let arr_reverse_decoded = decode_with_termination(&arr_reverse_encoded);
    let rev_arr_check: Vec<u8> = arr.into_iter().rev().collect();
    println!(
        "Result: {:?}\nFrom: {:?}",
        &arr_reverse_decoded.as_ref().unwrap(),
        arr
    );
    println!(
        "Module reversed array: {:?}",
        &arr_reverse_decoded.unwrap() == &rev_arr_check
    );

    // TODO!: Deallocate mem

    /*
       // -------------
       // hello world string
       // -------------

       // subject to bytes
       let subject = "Wasmer";
       // length to allocate
       let subject_length = subject.len();

       /*     // get pointer as boxed wasmer value & allocate
       let input_pointer = allocate.call(&[Value::I32(subject_length as i32)]).unwrap();
       // unboxed wasmer value
       let input_pointer_unpacked = &input_pointer[0]; */
       // convert back to native usize
       let ptr_in = &allocate.call(&[Value::I32(subject_length as i32)]).unwrap()[0];
       let ptr_in = Value::unwrap_i32(&ptr_in) as usize;

       // https://radu-matei.com/blog/practical-guide-to-wasm-memory/
       let memory = instance.exports.get_memory("memory").unwrap();
       // https://github.com/wasmerio/wasmer-rust-example/blob/master/examples/string.rs
       // write into linear memory
       for (byte, cell) in subject
           .bytes()
           .zip(memory.view()[ptr_in as usize..ptr_in + subject_length].iter())
       {
           cell.set(byte);
       }

       _debug_print_mem_bytes(&memory, ptr_in, subject_length);

       // debug: rebuild string from module memory
       let in_vec: Vec<u8> = memory.view()[ptr_in as usize..(ptr_in + subject_length)]
           .iter()
           .map(|cell| cell.get())
           .collect();

       println!("str_vec: {:?}", in_vec);
       // Convert the subslice to a `&str`.
       let string = std::str::from_utf8(&in_vec).unwrap();

       println!("inputstring from mem: {:?}", string);

       let output = greet.call(&[Value::I32(ptr_in as i32)]).unwrap();

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
    */
}

fn read_until_sequence(memory: &wasmer::Memory, ptr: usize) -> Vec<u8> {
    let mem_view = memory.view();
    let mut sequence: Vec<u8> = Vec::new();

    // read mem cells until termination sequence base64-helper::B64_TERMINATION_SEQ is found
    // or i exceeds max_sequence length to prevent infinite loops
    let mut i: usize = 0;
    while i <= get_max_length() as usize {
        let val = mem_view[ptr + i].get();
        sequence.push(val);

        let sequence_iter = sequence.to_vec();
        let term_sequ: Vec<u8> = sequence_iter.into_iter().rev().take(2).collect();
        let term_sequ: Vec<u8> = term_sequ.into_iter().rev().collect();

        if term_sequ == get_termination_sequence().as_bytes() {
            println!(
                "Success -- Checking: {:?}, Found: {:?}",
                get_termination_sequence().as_bytes(),
                term_sequ
            );
            break;
        }
        i += 1;
    }

    sequence
}

fn _debug_print_mem_bytes(memory: &wasmer::Memory, ptr: usize, len: usize) {
    let view: MemoryView<u8> = memory.view();
    println!("{0: <30}", "\n______________________________");
    println!("{0: <12} | {1: <6} | {2: <6}", "ADDRESS", "VALUE", "UTF-8");
    println!(
        "{0: <12} | {1: <6} | {2: <6}",
        "----------", "------", "------"
    );
    for (i, cell) in view[ptr..ptr + len].iter().enumerate() {
        let tempcell = &[cell.get()];
        let utf8val = match std::str::from_utf8(tempcell) {
            Ok(v) => v,
            Err(e) => "!=",
        };
        println!(
            "0x{0: <10} | {1: <6} | {2: <6}",
            ptr + i,
            cell.get(),
            utf8val
        );
    }
    println!("{0: <30}", "______________________________\n");
}
