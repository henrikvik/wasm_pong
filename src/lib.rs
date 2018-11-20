extern crate console_error_panic_hook;
extern crate nalgebra as na;
extern crate wasm_bindgen;
extern crate web_sys;

use js_sys::WebAssembly;
use std::f32;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGl2RenderingContext as GL;
use web_sys::{Document, Window};
use web_sys::{HtmlCanvasElement, WebGlBuffer, WebGlProgram, WebGlShader, WebGlUniformLocation};

static VERTEX_SOURCE: &str = "
attribute vec4 vertex_position;
attribute vec4 vertex_color;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

varying lowp vec4 color;

void main() {
    gl_Position = projection * view * model * vertex_position;
    color = vertex_color;
}";

static FRAGMENT_SOURCE: &str = "
varying lowp vec4 color;

void main() {
    gl_FragColor = color;
}";

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

    renderer: Renderer,

    i: f32,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> Result<App, JsValue> {
        console_error_panic_hook::set_once();

        let width = 400;
        let height = 400;

        let window = web_sys::window().expect("no window");
        let document = window.document().expect("no document");
        Self::add_style(
            &document,
            "html { 
                background: #DDD;
                margin: 0;
                width: 100%;
                height: 100%;
                display: flex;
                justify-content: center;
                align-items: center;
            }",
        );

        let canvas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;

        canvas.set_width(width);
        canvas.set_height(height);
        document.body().expect("no body").append_child(&canvas)?;

        let gl: GL = canvas.get_context("webgl2")?.unwrap().dyn_into()?;

        Ok(App {
            window,
            document,
            canvas,
            renderer: Renderer::new(gl)?,
            i: 0.0,
        })
    }

    pub fn update(&mut self) {
        // log(&format!("{}",self.i));

        self.i += 0.01;

        self.renderer.clear();
        self.renderer.draw();
    }

    fn add_style(document: &Document, css: &str) {
        let style = document.create_element("style").unwrap();
        style.set_inner_html(css);
        document.head().unwrap().append_child(&style).unwrap();
    }
}

struct Renderer {
    gl: GL,
    program: ShaderProgram,
    buffers: Buffers,
    camera: Camera,
}

impl Renderer {
    fn new(gl: GL) -> Result<Renderer, JsValue> {
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear_depth(1.0);
        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LEQUAL);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let program = ShaderProgram::new(&gl, VERTEX_SOURCE, FRAGMENT_SOURCE)?;
        program.use_program(&gl);

        let buffers = Buffers {
            position: Buffer::new(
                &gl,
                program.attributes.position,
                2,
                Box::new([
                    -1.0,  1.0, 
                     1.0,  1.0, 
                    -1.0, -1.0, 
                     1.0, -1.0
                ]),
            ),
            color: Buffer::new(
                &gl,
                program.attributes.color,
                4,
                Box::new([
                    1.0, 0.0, 0.0, 1.0, 
                    0.0, 1.0, 0.0, 1.0, 
                    0.0, 0.0, 1.0, 1.0, 
                    1.0, 1.0, 1.0, 1.0,
                ]),
            ),
        };


        let camera = Camera::new(
            400.0 / 400.0, 
            45.0,
            na::Point3::new(0.0, 0.0, 5.0),
            na::Point3::new(0.0, 0.0, 0.0),
        );


        gl.uniform_matrix4fv_with_f32_array(
            Some(&program.uniforms.projection),
            false,
            // na::Matrix4::<f32>::identity().as_mut_slice(),
            camera.get_projection().as_mut_slice(),
        );

        gl.uniform_matrix4fv_with_f32_array(
            Some(&program.uniforms.view),
            false,
            // na::Matrix4::<f32>::identity().as_mut_slice(),
            camera.get_view().as_mut_slice(),
        );

        gl.uniform_matrix4fv_with_f32_array(
            Some(&program.uniforms.model),
            false,
            na::Matrix4::<f32>::identity().as_mut_slice(),
        );

        Ok(Renderer {
            gl,
            program,
            buffers,
            camera,
        })
    }

    fn clear(&self) {
        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
    }

    fn draw(&self) {
        self.gl.draw_arrays(GL::TRIANGLE_STRIP, 0, 4);
    }
}

struct ShaderProgram {
    program: WebGlProgram,
    attributes: AttributeLocations,
    uniforms: UniformLocations,
}

impl ShaderProgram {
    fn new(gl: &GL, vertex_source: &str, fragment_source: &str) -> Result<ShaderProgram, JsValue> {
        let vertex_shader = Self::load_shader(&gl, GL::VERTEX_SHADER, vertex_source)?;
        let fragment_shader = Self::load_shader(&gl, GL::FRAGMENT_SHADER, fragment_source)?;

        let program = Self::link_program(&gl, &vertex_shader, &fragment_shader)?;

        let position = gl.get_attrib_location(&program, "vertex_position") as u32;

        let color = gl.get_attrib_location(&program, "vertex_color") as u32;

        let attributes = AttributeLocations { position, color };

        let model = gl
            .get_uniform_location(&program, "model")
            .expect("uniform not found");
        let view = gl
            .get_uniform_location(&program, "view")
            .expect("uniform not found");
        let projection = gl
            .get_uniform_location(&program, "projection")
            .expect("uniform not found");

        let uniforms = UniformLocations {
            model,
            view,
            projection,
        };

        Ok(ShaderProgram {
            program,
            attributes,
            uniforms,
        })
    }

    fn use_program(&self, gl: &GL) {
        gl.use_program(Some(&self.program));
        gl.enable_vertex_attrib_array(self.attributes.position);
        gl.enable_vertex_attrib_array(self.attributes.color);
    }

    fn load_shader(gl: &GL, shader_type: u32, shader_source: &str) -> Result<WebGlShader, JsValue> {
        let shader = gl
            .create_shader(shader_type)
            .expect("failed to create shader");

        gl.shader_source(&shader, shader_source);
        gl.compile_shader(&shader);

        if gl
            .get_shader_parameter(&shader, GL::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            let err = gl
                .get_shader_info_log(&shader)
                .unwrap_or("unknow shader compile error".into());
            gl.delete_shader(Some(&shader));
            Err(err.into())
        }
    }

    fn link_program(
        gl: &GL,
        vertex_shader: &WebGlShader,
        fragment_shader: &WebGlShader,
    ) -> Result<WebGlProgram, String> {
        let program = gl.create_program().unwrap();

        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        if gl
            .get_program_parameter(&program, GL::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            let err = gl
                .get_program_info_log(&program)
                .unwrap_or("unknow program error".into());
            gl.delete_program(Some(&program));
            Err(err.into())
        }
    }
}

struct AttributeLocations {
    position: u32,
    color: u32,
}

struct UniformLocations {
    model: WebGlUniformLocation,
    view: WebGlUniformLocation,
    projection: WebGlUniformLocation,
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

struct Camera {
    view: na::Isometry3<f32>,
    projection: na::Perspective3<f32>,
}

impl Camera {
    fn new(aspect : f32, fov : f32, eye: na::Point3<f32>, target: na::Point3<f32>) -> Camera {
        let projection = na::Perspective3::new(aspect, Self::deg_to_rad(fov), 0.01, 100.0);
        let view = na::Isometry3::look_at_rh(&eye, &target, &na::Vector3::y());

        // let eye    = na::Point3::new(0.0, 0.0, -1.0);
        // let target = na::Point3::new(0.0, 0.0, 0.0);
        // let view   = na::Isometry3::look_at_rh(&eye, &target, &na::Vector3::y());

        // let projection = na::Perspective3::new(16.0 / 9.0, 3.14 / 2.0, 1.0, 1000.0);


        Camera {
            view,
            projection,
        }
    }

    fn deg_to_rad(deg: f32) -> f32 {
        deg * f32::consts::PI / 180.0
    }

    fn get_view(&self) -> na::Matrix4<f32> {
        self.view.to_homogeneous()
    }

    fn get_projection(&self) -> na::Matrix4<f32> {
        self.projection.to_homogeneous()
    }
}
