use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGlBuffer;
use web_sys::WebGl2RenderingContext as GL;

pub(super) struct Buffer {
    buffer: WebGlBuffer,
}

impl Buffer {
    pub fn new(gl: &GL) -> Buffer {
        let buffer = gl.create_buffer().unwrap();
        Buffer { buffer }
    }

    pub fn set_attribute_ptr(&self, gl : &GL, location: u32, num_components: i32) {
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffer));
        gl.vertex_attrib_pointer_with_i32(location, num_components, GL::FLOAT, false, 0, 0);
        gl.bind_buffer(GL::ARRAY_BUFFER, None);
    }

    pub fn upload_data(&self, gl : &GL, data : &[f32]) {
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffer));
        gl.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &js_array_wrapper(data.as_ref()),
            GL::STATIC_DRAW,
        );
        gl.bind_buffer(GL::ARRAY_BUFFER, None);
    }
}

fn js_array_wrapper(array: &[f32]) -> js_sys::Float32Array {
    let array_location = array.as_ptr() as u32 / 4; // devided by 4 ????

    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();

    js_sys::Float32Array::new(&memory_buffer).subarray(
        //// WHO OWNS THIS?
        array_location,
        array_location + array.len() as u32,
    )
}