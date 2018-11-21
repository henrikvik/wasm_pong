extern crate nalgebra as na;

use super::{ShaderProgram, Camera};

use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as GL;
use web_sys::WebGlBuffer;

static VERTEX_SOURCE: &str = "
attribute vec4 vertex_position;
attribute vec4 vertex_color;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

varying lowp vec4 color;

void main() {
    gl_Position = projection * view * model * vertex_position;
    // gl_Position = vertex_position;
    color = vertex_color;
}";

static FRAGMENT_SOURCE: &str = "
varying lowp vec4 color;

void main() {
    gl_FragColor = color;
    // gl_FragColor = vec4(1.0,1.0,1.0,1.0);
}";

pub struct Renderer {
    gl: GL,
    program: ShaderProgram,
    buffers: Buffers,
    camera: Camera,
    i: f32,
}

impl Renderer {
    pub fn new(gl: GL) -> Renderer {
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear_depth(1.0);
        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LEQUAL);

        let mut program = ShaderProgram::new(&gl, VERTEX_SOURCE, FRAGMENT_SOURCE);

        program.register_attribute(&gl, "vertex_position");
        program.register_attribute(&gl, "vertex_color");
        program.register_uniform(&gl, "projection");
        program.register_uniform(&gl, "model");
        program.register_uniform(&gl, "view");

        let buffers = Buffers {
            position: Buffer::new(
                &gl,
                program.get_attribute("vertex_position"),
                2,
                Box::new([-1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0]),
            ),
            color: Buffer::new(
                &gl,
                program.get_attribute("vertex_color"),
                4,
                Box::new([
                    1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
                ]),
            ),
        };

        program.use_program(&gl);

        let camera = Camera::new(
            400.0 / 400.0,
            45.0,
            na::Point3::new(0.0, 0.0, 5.0),
            na::Point3::new(0.0, 0.0, 0.0),
        );

        program.upload_uniform(&gl, "projection", &mut camera.get_projection());
        program.upload_uniform(&gl, "view", &mut camera.get_view());

        Renderer {
            i: 0.0,

            gl,
            program,
            buffers,
            camera,
        }
    }

    pub fn clear(&self) {
        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
    }

    pub fn draw(&mut self) {
        self.clear();
        self.i += 0.05;

        self.program.use_program(&self.gl);

        self.program.upload_uniform(
            &self.gl,
            "model",
            &mut na::Matrix4::<f32>::from_euler_angles(0.6, self.i, 0.0),
        );

        self.gl.draw_arrays(GL::TRIANGLE_STRIP, 0, 4);
    }
}

struct Buffers {
    position: Buffer,
    color: Buffer,
}

struct Buffer {
    data: Box<[f32]>,
    buffer: WebGlBuffer,
}

impl Buffer {
    fn new(gl: &GL, location: u32, num_components: i32, data: Box<[f32]>) -> Buffer {
        let buffer = gl.create_buffer().unwrap();

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        gl.vertex_attrib_pointer_with_i32(location, num_components, GL::FLOAT, false, 0, 0);
        gl.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &Self::js_array_wrapper(data.as_ref()),
            GL::STATIC_DRAW,
        );

        Buffer { buffer, data }
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
}
