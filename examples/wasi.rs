use anyhow::Result;
use wasi_common::sync::WasiCtxBuilder;
use wasmtime::*;

fn main() -> Result<()> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let mut linker: Linker<wasi_common::WasiCtx> = Linker::new(&engine);
    // wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
    wasi_common::sync::add_to_linker(&mut linker, |s| s)?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, wasi);
    let wasm_file = "/Users/davidviejo/poc/test-python/wasmer_server/testwasmtime/hello-wasm/target/wasm32-wasi/release/hello-wasm.wasm";

    // Instantiate our module with the imports we've created, and run it.
    let module = Module::from_file(&engine, wasm_file)?;

    println!("Available functions:");
    for export in module.exports() {
        if export.ty().func().is_some() {
            println!("  - {}", export.name());
        } else if export.ty().memory().is_some() {
            println!("Memory - {}", export.name());
        }
    }

    let instance = linker.instantiate(&mut store, &module)?;

    // Get the function from the module
    let hello_endpoint_c = instance.get_typed_func::<(), i32>(&mut store, "hello_endpoint_c")?;

    // Get the memory
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow::anyhow!("Failed to find memory export"))?;

    // Call the function
    let result_ptr = hello_endpoint_c.call(&mut store, ())?;

    // Read the pointer and length from memory
    let mut buffer: [u64; 2] = [0u64; 2];
    memory.read(
        &mut store,
        result_ptr as usize,
        bytemuck::cast_slice_mut(&mut buffer),
    )?;
    let [string_ptr, string_len] = buffer;

    // Read the actual string data
    let mut string_buffer = vec![0u8; string_len as usize];
    memory.read(&mut store, string_ptr as usize, &mut string_buffer)?;

    // Convert buffer to string
    let string = String::from_utf8(string_buffer)?;
    println!("Result string: {}", string);

    // Free the result in WebAssembly memory
    let free_result = instance.get_typed_func::<i32, ()>(&mut store, "free_result")?;
    free_result.call(&mut store, result_ptr)?;

    Ok(())
}
