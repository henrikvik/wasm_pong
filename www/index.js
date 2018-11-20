let wasm = import("../target/webpack/wasm_pong");
wasm.then(wasm => {

    try {

        let app = wasm.App.new();

        let loop = ()=>{
            app.update();
            requestAnimationFrame(loop);
        }

        loop();

    } catch (e) {
        console.error(e);
    }

});