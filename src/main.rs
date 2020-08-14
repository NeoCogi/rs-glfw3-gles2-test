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
use rs_mem::*;
use rs_streams::*;

mod objloader;
use objloader::*;

#[cfg(not(test))]
#[panic_handler]
fn alt_std_panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { libc::exit(1) }
}

fn load_shader(src: &str, ty: GLenum) -> Option<GLuint> {
    unsafe {
        let shader = glCreateShader(ty);
        if shader == 0 {
            return None
        }

        glShaderSource(shader, 1, &(src.as_ptr() as *const i8), core::ptr::null());
        glCompileShader(shader);

        let mut compiled = 0;
        glGetShaderiv(shader, GL_COMPILE_STATUS, &mut compiled);
        if compiled == 0 {
            let mut info_len = 0;
            glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut info_len);
            if info_len > 1 {
                let sptr = alloc_array::<u8>(info_len as usize);
                glGetShaderInfoLog(shader, info_len as GLsizei, core::ptr::null_mut(), sptr as *mut GLchar);
                libc::puts(sptr as *const i8);
                free_array(sptr, info_len as usize, info_len as usize);
            }

            glDeleteShader(shader);
            return None
        }
        Some(shader)
    }
}

fn load_program(vs: &str, fs: &str) -> Option<GLuint> {
    unsafe {
        let program_object = glCreateProgram();
        if program_object == 0 {
            return None
        }

        let vertex_shader    = load_shader(vs, GL_VERTEX_SHADER);
        let fragment_shader  = load_shader(fs, GL_FRAGMENT_SHADER);

        match (vertex_shader, fragment_shader) {
            (None, None) => (),
            (None, Some(f)) => glDeleteShader(f),
            (Some(v), None) => glDeleteShader(v),
            (Some(v), Some(f)) => {
                glAttachShader(program_object, v);
                glAttachShader(program_object, f);
                glBindAttribLocation(program_object, 0, "vPosition\0".as_ptr() as *const GLchar);
                glLinkProgram(program_object);

                let mut linked = 0;
                glGetProgramiv(program_object, GL_LINK_STATUS, &mut linked);
                if linked == 0 {
                    let mut info_len = 0;
                    glGetProgramiv(program_object, GL_INFO_LOG_LENGTH, &mut info_len);
                    if info_len > 1 {
                        let sptr = alloc_array::<u8>(info_len as usize);
                        glGetProgramInfoLog(program_object, info_len as GLsizei, core::ptr::null_mut(), sptr as *mut GLchar);
                        libc::puts(sptr as *const i8);
                        free_array(sptr, info_len as usize, info_len as usize);
                    }

                    glDeleteShader(program_object);
                    return None
                }
            }
        }

        Some(program_object)
    }
}

static VERTEX_SHADER : &'static str = "
attribute highp vec4 vPosition;
void main() {
    gl_Position = vPosition;
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
    program : Option<GLuint>,
    buff    : GLuint,
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

        match (*state).program {
            Some(p) => {

                glUseProgram(p);
                glBindBuffer(GL_ARRAY_BUFFER, (*state).buff);
                glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE as u8, 0, 0 as *const c_void);
                glEnableVertexAttribArray(0);
                glDrawArrays(GL_TRIANGLES, 0, 3);
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

        let program = load_program(&VERTEX_SHADER, &FRAGMENT_SHADER);

        let m = Mesh::read_obj("suzane.obj");
        match m {
            Ok(m) => println!("verts     : {}\nuvws      : {}\ntris      : {}\nquads     : {}", m.verts().len(), m.uvws().len(), m.tris().len(), m.quads().len()),
            _ => ()
        }
        let mut buff = 0;
        glGenBuffers(1, &mut buff);
        let vertices : [f32; 9] =
        [   0.0,    0.5,    0.0,
            -0.5,   -0.5,   0.0,
            0.5,    -0.5,   0.0 ];
        glBindBuffer(GL_ARRAY_BUFFER, buff);
        glBufferData(GL_ARRAY_BUFFER, 4 * 9 as GLsizeiptr, vertices.as_ptr() as *const rs_ctypes::c_void, GL_STATIC_DRAW);

        let state = Box::new(State { program : program, buff : buff});
        glfwSetWindowUserPointer(win, state.as_ref() as *const State as *mut ::core::ffi::c_void);
        run_main_loop(win);

        glfwDestroyWindow(win);
        glfwTerminate();
    }
    0
}