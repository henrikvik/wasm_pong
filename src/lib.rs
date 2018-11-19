extern crate wasm_bindgen;
extern crate web_sys;
extern crate nalgebra as na;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlProgram, WebGl2RenderingContext, WebGlShader, WebGlBuffer};
use web_sys::WebGl2RenderingContext as gl;
use std::f32;



#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
    let width = 400;
    let height = 400;

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");
    add_style(&document, "
        html { 
            background: #DDD;
            margin: 0;
            width: 100%;
            height: 100%;
            display: flex;
            justify-content: center;
            align-items: center;
        }
    ");

    let canvas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
    canvas.set_width(width);
    canvas.set_height(height);
    document.body().expect("no body").append_child(&canvas)?;

    let context: WebGl2RenderingContext = canvas.get_context("webgl2")?.unwrap().dyn_into()?;

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear_depth(1.0);
    context.enable(gl::DEPTH_TEST);
    context.depth_func(gl::LEQUAL);
    context.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

    let vs_source = "
        attribute vec4 vertex_position;
        attribute vec4 vertex_color;

        uniform mat4 model_view_matrix;
        uniform mat4 projection_matrix;

        varying lowp vec4 color;

        void main() {
            gl_Position = projection_matrix * model_view_matrix * vertex_position;
            color = vertex_color;
        }
    ";

    let fs_source = "
        varying lowp vec4 color;

        void main() {
            gl_FragColor = color;
        }
    ";

    let vertex_shader   = load_shader(&context, gl::VERTEX_SHADER, vs_source)?;
    let fragment_shader = load_shader(&context, gl::FRAGMENT_SHADER, fs_source)?;
    let program         = link_shaders(&context, &vertex_shader, &fragment_shader)?;


    let vertex_position_location   = context.get_attrib_location(&program, "vertex_position") as u32;
    let vertex_color_location      = context.get_attrib_location(&program, "vertex_color") as u32;
    let model_view_matrix_location = context.get_uniform_location(&program, "model_view_matrix").expect("uniform not found");
    let projection_matrix_location = context.get_uniform_location(&program, "projection_matrix").expect("uniform not found");

    let buffers = init_buffers(&context);

    let projection = na::Perspective3::new(
        canvas.width() as f32 / canvas.height() as f32, 
        45.0 * f32::consts::PI / 180.0, 
        0.1, 
        100.0
    );

    let model = na::Isometry3::<f32>::new(na::zero(), na::zero());
    let view = na::Isometry3::<f32>::look_at_rh(
        &na::Point3::new(0.0, 0.0, 5.0), 
        &na::Point3::new(0.0, 0.0, 0.0), 
        &na::Vector3::y()
    );
    
    let model_view = view * model;

    {
        let num_components = 2;
        let data_type = gl::FLOAT;
        let normalize = false;
        let stride = 0;
        let offset = 0;
        context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffers.position));
        context.enable_vertex_attrib_array(vertex_position_location);
        context.vertex_attrib_pointer_with_i32(
            vertex_position_location,
            num_components,
            data_type,
            normalize,
            stride,
            offset
        );

        context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffers.color));
        context.enable_vertex_attrib_array(vertex_color_location);
        context.vertex_attrib_pointer_with_i32(
            vertex_color_location,
            4,
            gl::FLOAT,
            false,
            0,
            0
        );

    }

    context.use_program(Some(&program));

    context.uniform_matrix4fv_with_f32_array(
        Some(&model_view_matrix_location),
        false,
        model_view.to_homogeneous().as_mut_slice()
        // na::Matrix4::<f32>::identity().as_mut_slice()
    );

    context.uniform_matrix4fv_with_f32_array(
        Some(&projection_matrix_location),
        false,
        projection.to_homogeneous().as_mut_slice()
        // na::Matrix4::<f32>::identity().as_mut_slice()
    );


    context.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);



    while true {
        

        context.draw_arrays(gl::TRIANGLE_STRIP, 0, 4);
        context.uniform_matrix4fv_with_f32_array(
            Some(&model_view_matrix_location),
            false,
            (view * model).to_homogeneous().as_mut_slice()
            // na::Matrix4::<f32>::identity().as_mut_slice()
        );
    }


    Ok(())
}

fn add_style(document: &web_sys::Document, css: &str) {
    let style = document.create_element("style").unwrap();
    style.set_inner_html(css);
    document.head().unwrap().append_child(&style).unwrap();
}

fn load_shader(context : &WebGl2RenderingContext, shader_type : u32, shader_source : &str) -> Result<WebGlShader, String> {
    let shader = context.create_shader(shader_type).expect("failed to create shader");

    context.shader_source(&shader, shader_source);
    context.compile_shader(&shader);

    if context.get_shader_parameter(&shader, gl::COMPILE_STATUS).as_bool().unwrap_or(false) {
        Ok(shader)
    } else {
        let err = context.get_shader_info_log(&shader).unwrap_or("unknow shader compile error".into());
        context.delete_shader(Some(&shader));
        Err(err.into())
    }
}

fn link_shaders(context : &gl, vertex_shader : &WebGlShader, fragment_shader : &WebGlShader) -> Result<WebGlProgram, String> {
    let program : WebGlProgram = context.create_program().unwrap();
    context.attach_shader(&program, &vertex_shader);
    context.attach_shader(&program, &fragment_shader);
    context.link_program(&program);

    if context.get_program_parameter(&program, gl::LINK_STATUS).as_bool().unwrap_or(false) {
        Ok(program)
    } else {
        let err = context.get_program_info_log(&program).unwrap_or("unknow program error".into());
        context.delete_program(Some(&program));
        Err(err.into())
    }
}

struct Buffers {
    position : WebGlBuffer,
    color : WebGlBuffer,
}

fn init_buffers(context : &gl) -> Buffers {

    let positions : [f32;8] = [
        -1.0,  1.0,
         1.0,  1.0,
        -1.0, -1.0,
         1.0, -1.0
    ];

    let position_buffer = context.create_buffer().unwrap();
    context.bind_buffer(gl::ARRAY_BUFFER, Some(&position_buffer));
    context.buffer_data_with_array_buffer_view(
        gl::ARRAY_BUFFER, 
        &f32_array_to_js_array_wrapper(&positions), 
        gl::STATIC_DRAW
    );

    let colors : [f32;16] = [
        1.0, 0.0, 0.0, 1.0,
        0.0, 1.0, 0.0, 1.0,
        0.0, 0.0, 1.0, 1.0,
        1.0, 1.0, 1.0, 1.0,
    ];

    let color_buffer = context.create_buffer().unwrap();
    context.bind_buffer(gl::ARRAY_BUFFER, Some(&color_buffer));
    context.buffer_data_with_array_buffer_view(
        gl::ARRAY_BUFFER,
        &f32_array_to_js_array_wrapper(&colors),
        gl::STATIC_DRAW
    );


    Buffers { 
        position : position_buffer,
        color : color_buffer,
    }
}

fn f32_array_to_js_array_wrapper(array : &[f32]) -> js_sys::Float32Array {
    let array_location = array.as_ptr() as u32 / 4; // devided by 4 ????

    let memory_buffer = wasm_bindgen::memory()          //
        .dyn_into::<WebAssembly::Memory>().unwrap()     //
        .buffer();                                      //
                                                        //
    js_sys::Float32Array::new(&memory_buffer).subarray( //// WHO OWNS THIS?
        array_location,                                 //
        array_location + array.len() as u32             //
    )                                                   //
}