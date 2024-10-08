use anyhow::Result;
use wasi_common::sync::WasiCtxBuilder;
use wasmtime::*;

struct WasmInstance {
    instance: Instance,
    store: Store<wasi_common::WasiCtx>,
    memory: Memory,
}

impl WasmInstance {
    fn new(engine: &Engine, wasm_file: &str) -> Result<Self> {
        let mut linker: Linker<wasi_common::WasiCtx> = Linker::new(engine);
        wasi_common::sync::add_to_linker(&mut linker, |s| s)?;

        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();
        let mut store = Store::new(engine, wasi);
        let module = Module::from_file(engine, wasm_file)?;
        let instance = linker.instantiate(&mut store, &module)?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Failed to find memory export"))?;

        Ok(Self {
            instance,
            store,
            memory,
        })
    }

    fn call_function<P, R>(&mut self, name: &str, params: P) -> Result<R>
    where
        P: WasmParams,
        R: WasmRet,
        R: WasmResults,
    {
        let func = self
            .instance
            .get_typed_func::<P, R>(&mut self.store, name)?;
        func.call(&mut self.store, params)
    }

    fn read_string(&mut self, ptr: i32) -> Result<String> {
        let mut buffer: [u64; 2] = [0u64; 2];
        self.memory.read(
            &mut self.store,
            ptr as usize,
            bytemuck::cast_slice_mut(&mut buffer),
        )?;
        let [string_ptr, string_len] = buffer;

        let mut string_buffer = vec![0u8; string_len as usize];
        self.memory
            .read(&mut self.store, string_ptr as usize, &mut string_buffer)?;

        Ok(String::from_utf8(string_buffer)?)
    }

    fn free_result(&mut self, ptr: i32) -> Result<()> {
        self.call_function("free_result", ptr)
    }
}

fn main() -> Result<()> {
    let engine = Engine::default();
    let wasm_file = "/Users/davidviejo/poc/test-python/wasmer_server/testwasmtime/hello-wasm/target/wasm32-wasi/release/hello-wasm.wasm";

    let mut wasm_instance = WasmInstance::new(&engine, wasm_file)?;

    // Print available functions
    let module = Module::from_file(&engine, wasm_file)?;
    println!("Available functions:");
    for export in module.exports() {
        if export.ty().func().is_some() {
            println!("  - {}", export.name());
        } else if export.ty().memory().is_some() {
            println!("Memory - {}", export.name());
        }
    }

    // Call the function and get the result
    let result_ptr: i32 = wasm_instance.call_function("hello_endpoint_c", ())?;
    let result_string = wasm_instance.read_string(result_ptr)?;
    println!("Result string: {}", result_string);

    // Free the result in WebAssembly memory
    wasm_instance.free_result(result_ptr)?;

    Ok(())
}
