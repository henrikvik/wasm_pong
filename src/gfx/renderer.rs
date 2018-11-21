extern crate nalgebra as na;

use super::{Buffer, Camera, ShaderProgram};

use std::collections::HashMap;

use web_sys::WebGl2RenderingContext as GL;

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
    buffers: HashMap<String, Buffer>,
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

        let mut buffers = HashMap::new();
        buffers.insert("position".into(), {
            let buffer = Buffer::new(&gl);

            buffer.set_attribute_ptr(&gl, program.get_attribute("vertex_position"), 2);
            buffer.upload_data(
                &gl,
                &[
                    -1.0,  1.0, //
                     1.0,  1.0, //
                    -1.0, -1.0, //
                     1.0, -1.0, //
                ],
            );

            buffer
        });

        buffers.insert("color".into(), {
            let buffer = Buffer::new(&gl);

            buffer.set_attribute_ptr(&gl, program.get_attribute("vertex_color"), 4);
            buffer.upload_data(
                &gl,
                &[
                    1.0, 0.0, 0.0, 1.0, //
                    0.0, 1.0, 0.0, 1.0, //
                    0.0, 0.0, 1.0, 1.0, //
                    1.0, 1.0, 1.0, 1.0, //
                ],
            );

            buffer
        });

        let camera = Camera::new(
            400.0 / 400.0,
            45.0,
            na::Point3::new(0.0, 0.0, 5.0),
            na::Point3::new(0.0, 0.0, 0.0),
        );

        program.use_program(&gl);
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
