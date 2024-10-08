Compile wasm first:

```bash
cd hello-wasm
cargo build --target wasm32-wasi --release
docker buildx build --platform wasi/wasm -t hello-wasm .
docker run --runtime=io.containerd.wasmedge.v1 --platform=wasi/wasm hello-wasm
```
Run wasi example:
```bash
cd ..
cargo run --example wasi
```