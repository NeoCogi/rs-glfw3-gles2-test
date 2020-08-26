// Copyright 2020-Present (c) Raja Lehtihet & Wael El Oraiby
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software without
// specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//
#![no_std]
#![no_main]

use rs_ctypes::*;
use rs_glfw3::bindings::*;
use rs_gles2::bindings::*;
use rs_streams::*;
use rs_alloc::*;
use rs_math3d::*;


mod renderer;
mod objloader;
mod gles2_renderer;

use objloader::*;
use renderer::*;
use gles2_renderer::*;

#[cfg(not(test))]
#[panic_handler]
fn alt_std_panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { libc::exit(1) }
}


static VERTEX_SHADER : &'static str = "
attribute highp vec4 vPosition;
uniform highp mat4 uMVP;
void main() {
    gl_Position = uMVP * vPosition;
}\0";

static FRAGMENT_SHADER : &'static str = "
precision mediump float;
void main() {
    gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}\0";

#[cfg(target_arch = "wasm32")]
type EmArgCallbackFunc = extern "C" fn(*mut c_void);


#[cfg(target_arch = "wasm32")]
extern "C" {
    fn emscripten_set_main_loop_arg(func: EmArgCallbackFunc, arg: *mut c_void, fps: c_int, simulate_infinite_loop: c_int);
}

pub struct State {
    program : Option<Box<dyn Program>>,
    buff    : StaticVertexBuffer,

    monkey_vb   : StaticVertexBuffer,
    monkey_ib   : StaticIndexBuffer,
}

struct Uniforms {
    mvp         : Mat4f,
}


impl UniformBlock for Uniforms {
    fn descriptors() -> Vec<UniformDataDesc> {
        let mut v = Vec::new();
        v.push(UniformDataDesc::new(String::from("uMVP"), UniformDataType::Float4x4, 0, 0));
        v
    }
}

extern "C"
fn main_loop(win_: *mut c_void) {
    unsafe {
        let win = win_ as *mut GLFWwindow;
        let state = glfwGetWindowUserPointer(win) as *mut State;

        let mut width = 0;
        let mut height = 0;
        glfwGetWindowSize(win, &mut width, &mut height);
        glViewport(0, 0, width, height);
        glScissor(0, 0, width, height);
        glClearColor(0.0, 0.0, 0.0, 1.0);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);
        let u = Uniforms { mvp: Mat4f::identity() };

        match &(*state).program {
            Some(p) => {
                draw(p, &(*state).buff, &u);
                glDisable(GL_CULL_FACE);
                draw_indexed(p, &(*state).monkey_vb, &(*state).monkey_ib, &u);
            },
            None => ()
        }

        glfwSwapBuffers(win);
        glfwPollEvents();
    }
}

#[cfg(target_arch = "wasm32")]
fn run_main_loop(win: *mut GLFWwindow) {
    unsafe { emscripten_set_main_loop_arg(main_loop, win as *mut c_void, 0, 1) };
}

#[cfg(not(target_arch = "wasm32"))]
fn run_main_loop(win: *mut GLFWwindow) {
    unsafe {
        while glfwWindowShouldClose(win) == GLFW_FALSE as c_int && glfwGetKey(win, GLFW_KEY_ESCAPE as c_int) == 0 {
            main_loop(win as *mut c_void);
        }
    }
}


#[link(name="c")]
#[no_mangle]
pub extern "C"
fn main(_argc: isize, _argv: *const *const u8) -> isize  {
    unsafe {
        if glfwInit() == GLFW_FALSE as c_int {
            return 1;
        }

        glfwWindowHint(GLFW_CONTEXT_CREATION_API as c_int, GLFW_EGL_CONTEXT_API as c_int);
        glfwWindowHint(GLFW_CLIENT_API as c_int, GLFW_OPENGL_ES_API as c_int);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR as c_int, 2);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR as c_int, 0);
        glfwWindowHint(GLFW_SAMPLES as c_int, 8);
        glfwWindowHint(GLFW_ALPHA_BITS as c_int, 0);

        let win = glfwCreateWindow(1024, 900,
            "App\0".as_bytes().as_ptr() as *const u8 as *const i8,
            core::ptr::null::<GLFWmonitor>() as *mut GLFWmonitor,
            core::ptr::null::<GLFWwindow>() as *mut GLFWwindow);
        glfwMakeContextCurrent(win);

        let attribs = [ VertexAttribute::new(String::from("vPosition"), VertexFormat::Float3, 0) ];
        let uniforms = [ UniformDesc::new(String::from("uMVP"), UniformDataType::Float4x4, 0) ];
        let program = GLProgram::load_program(&VERTEX_SHADER, &FRAGMENT_SHADER, &attribs, &uniforms);

        let m =
            match Mesh::read_obj("suzane.obj") {
                Ok(m) => {
                    println!("verts     : {}\nuvws      : {}\ntris      : {}\nquads     : {}", m.verts().len(), m.uvws().len(), m.tris().len(), m.quads().len());
                    GPUMesh::from(&m)
                },
                _ => panic!("Error reading file")
            };

        let monkey_vb = StaticVertexBuffer::new(m.verts());
        let monkey_ib = StaticIndexBuffer::new(m.tris());
        let vertices : [Vec3f; 3] =
        [   Vec3f::new(0.0,    0.5,    0.0),
            Vec3f::new(-0.5,   -0.5,   0.0),
            Vec3f::new(0.5,    -0.5,   0.0) ];

        let buff = StaticVertexBuffer::new(&vertices);

        let state = Box::new(State { program : program, buff : buff, monkey_vb: monkey_vb, monkey_ib: monkey_ib });
        glfwSetWindowUserPointer(win, state.as_ref() as *const State as *mut ::core::ffi::c_void);
        run_main_loop(win);

        glfwDestroyWindow(win);
        glfwTerminate();
    }
    0
}