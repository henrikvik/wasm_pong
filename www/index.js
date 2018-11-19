let wasm = import("../target/webpack/wasm_pong");
wasm.then(wasm => wasm.main());