FROM scratch
COPY target/wasm32-wasi/release/hello-wasm.wasm /hello-wasm.wasm
ENTRYPOINT [ "/hello-wasm.wasm" ]
