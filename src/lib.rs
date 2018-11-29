#[macro_use]
extern crate lazy_static;

extern crate console_error_panic_hook;
extern crate wasm_bindgen;
extern crate web_sys;

mod gfx;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Window, HtmlCanvasElement, WebGl2RenderingContext};



#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct App {
    window: Window,
    document: Document,
    canvas: HtmlCanvasElement,

    renderer: gfx::Renderer,

    i: f32,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> App {
        console_error_panic_hook::set_once();

        let width = 400;
        let height = 400;

        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");

        Self::add_style(
            &document,
            "html {
                height: 100%;
            }

            body {
                background: #DDD;
                margin: 0;
                width: 100%;
                height: 100%;
                display: flex;
                justify-content: center;
                align-items: center;
            }

            canvas {
                max-width: 100%;
                max-height: 100%;
            }
            ",
        );

        let canvas: HtmlCanvasElement = document
            .create_element("canvas").expect("failed to create canvas")
            .dyn_into().expect("into failed");

        canvas.set_width(width);
        canvas.set_height(height);
        document.body().expect("no body")
            .append_child(&canvas).expect("failed to append");

        let gl: WebGl2RenderingContext = canvas
            .get_context("webgl2").expect("failed to get context").unwrap()
            .dyn_into().expect("into failed");

        App {
            window,
            document,
            canvas,
            renderer: gfx::Renderer::new(gl),
            i: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.i += 0.01;
        self.renderer.draw();
    }

    fn add_style(document: &Document, css: &str) {
        let style = document.create_element("style").unwrap();
        style.set_inner_html(css);
        document.head().unwrap().append_child(&style).unwrap();
    }
}
