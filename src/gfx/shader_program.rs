use std::collections::HashMap;
use std::sync::Mutex;
use web_sys::WebGl2RenderingContext as GL;
use web_sys::{WebGlProgram, WebGlShader, WebGlUniformLocation};

lazy_static! {
    static ref ACTIVE_UID: Mutex<u32> = Mutex::new(0);
}

fn get_active_uid() -> u32 {
    *ACTIVE_UID.lock().unwrap()
}

fn set_active_uid(uid: u32) {
    *ACTIVE_UID.lock().unwrap() = uid;
}

fn get_next_uid() -> u32 {
    lazy_static! {
        static ref NEXT_UID: Mutex<u32> = Mutex::new(0);
    }
    let mut next_uid = NEXT_UID.lock().unwrap();
    *next_uid += 1;
    *next_uid
}

pub(super) struct ShaderProgram {
    uid: u32,

    program: WebGlProgram,
    attributes: HashMap<String, u32>,
    uniforms: HashMap<String, WebGlUniformLocation>,
}

impl ShaderProgram {
    pub fn new(gl: &GL, vertex_source: &str, fragment_source: &str) -> ShaderProgram {
        let vertex_shader = compile_shader(&gl, GL::VERTEX_SHADER, vertex_source)
            .expect("vertex shader compile error");
        let fragment_shader = compile_shader(&gl, GL::FRAGMENT_SHADER, fragment_source)
            .expect("fragment shader compile error");

        let program = link_program(&gl, &vertex_shader, &fragment_shader).unwrap();

        ShaderProgram {
            uid: get_next_uid(),
            program,
            uniforms: HashMap::new(),
            attributes: HashMap::new(),
        }
    }

    pub fn use_program(&self, gl: &GL) {
        gl.use_program(Some(&self.program));

        for (_, location) in self.attributes.iter() {
            gl.enable_vertex_attrib_array(location.clone());
        }

        set_active_uid(self.uid);
    }

    pub fn register_attribute(&mut self, gl: &GL, name: &str) {
        let location = gl.get_attrib_location(&self.program, name);
        if location >= 0 {
            self.attributes.insert(String::from(name), location as u32);
        } else {
            panic!("attribute '{}' does not exist");
        }
    }

    pub fn register_uniform(&mut self, gl: &GL, name: &str) {
        let location = gl
            .get_uniform_location(&self.program, name)
            .expect(&format!("uniform '{}' does not exist", name));

        self.uniforms.insert(String::from(name), location);
    }

    pub fn get_attribute(&self, name: &str) -> u32 {
        self.attributes
            .get(name)
            .expect(&format!("attribute '{}' not registered", name))
            .clone()
    }

    pub fn upload_uniform(&self, gl: &GL, name: &str, payload: &mut UniformPayload) {
        assert_eq!(self.uid, get_active_uid());

        let location = self
            .uniforms
            .get(name)
            .expect(&format!("attribute '{}' not registered", name));

        payload.upload(&gl, &location);
    }
}

fn compile_shader(gl: &GL, shader_type: u32, shader_source: &str) -> Result<WebGlShader, String> {
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
            .unwrap_or(String::from("unknow error"));
        gl.delete_shader(Some(&shader));
        Err(err)
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
            .unwrap_or(String::from("unknow error"));
        gl.delete_program(Some(&program));
        Err(err)
    }
}

use nalgebra as na;

pub(super) trait UniformPayload {
    fn upload(&mut self, gl: &GL, location: &WebGlUniformLocation);
}

impl UniformPayload for na::Matrix4<f32> {
    fn upload(&mut self, gl: &GL, location: &WebGlUniformLocation) {
        gl.uniform_matrix4fv_with_f32_array(Some(&location), false, self.as_mut_slice());
    }
}
