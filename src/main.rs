Copyright 2020-Present (c) Raja Lehtihet & Wael El Oraiby
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
#![allow(non_snake_case)]
use rs_ctypes::*;
use rs_glfw3::bindings::*;
use rs_gles2::bindings::*;

pub unsafe fn allocRaw(size: usize) -> *mut u8 {
    //let addr = libc::memalign(core::mem::size_of::<usize>(), size) as *mut u8;
    let addr = libc::calloc(core::mem::size_of::<usize>(), size) as *mut u8;
    //libc::memset(addr as *mut libc::c_void, 0, size);
    addr
}

pub unsafe fn freeRaw(arr: *mut u8) {
    libc::free(arr as *mut libc::c_void);
}

pub unsafe fn alloc<T>() -> *mut T {
    allocRaw(core::mem::size_of::<T>()) as *mut T
}

pub unsafe fn free<T>(t: *mut T) {
    freeRaw(t as *mut u8)
}

// TODO: change this to const generics when they become stable and return a slice
pub unsafe fn allocArray<T>(count: usize) -> *mut T {
    allocRaw(core::mem::size_of::<T>() * count) as *mut T
}

// TODO: change this to slice once const generics stable
pub unsafe fn freeArray<T>(ptr: *mut T, count: usize) {
    let arr      = core::slice::from_raw_parts_mut(ptr, count); // this will keep a pointer (will not free it)
    for i in 0..count {
        ::core::ptr::drop_in_place(&arr[i] as *const T as *mut T);
    }
    free(ptr);
}

#[repr(C)]
pub struct Unique<T: ?Sized> {
    ptr         : *mut T,
    _marker     : ::core::marker::PhantomData<T>,
}

impl<T> Unique<T> {
    pub fn new(ptr: *mut T) -> Self { Self { ptr : ptr, _marker: ::core::marker::PhantomData } }
    pub fn getMutPtr(&mut self) -> *mut T { self.ptr }
    pub fn getPtr(&self) -> *const T { self.ptr }
}

#[repr(C)]
pub struct Box<T>{
    uptr: Unique<T>
}

impl<T> Box<T> {
    /// Allocates memory on the heap and then places `x` into it.
    ///
    /// # Examples
    ///
    /// ```
    /// let five = Box::new(5);
    /// ```
    #[inline(always)]
    pub fn new(x: T) -> Box<T> {
        unsafe {
            let addr = alloc::<T>();
            *addr = x;
            Self { uptr: Unique::new(addr) }
        }
    }

    pub fn asRef(&self) -> &T { unsafe { &(*self.uptr.getPtr()) } }
    pub fn asMut(&mut self) -> &T { unsafe { &mut (*self.uptr.getMutPtr()) } }
    pub fn intoRaw(self) -> *mut T {
        let m = ::core::mem::ManuallyDrop::new(self);
        m.uptr.ptr
    }

    pub fn fromRaw(raw: *mut T) -> Self {
        Self { uptr: Unique::new(raw) }
    }

    pub fn unbox(self) -> T {
        unsafe {
            let ptr = self.uptr.ptr;
            let v = self.intoRaw().read();
            free(ptr);
            v
        }
    }
}

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            let addr = self.uptr.getMutPtr();
            ::core::ptr::drop_in_place(addr);
            free(addr);
        }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn alt_std_panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { libc::exit(1); }
}

fn loadShader(src: &str, ty: GLenum) -> Option<GLuint> {
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
            let mut infoLen = 0;
            glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut infoLen);
            if infoLen > 1 {
                let sptr = allocArray::<u8>(infoLen as usize);
                glGetShaderInfoLog(shader, infoLen as GLsizei, core::ptr::null_mut(), sptr as *mut GLchar);
                libc::puts(sptr as *const i8);
                freeArray(sptr, infoLen as usize);
            }

            glDeleteShader(shader);
            return None
        }
        Some(shader)
    }
}

fn loadProgram(vs: &str, fs: &str) -> Option<GLuint> {
    unsafe {
        let programObject = glCreateProgram();
        if programObject == 0 {
            return None
        }

        let vertexShader = loadShader(vs, GL_VERTEX_SHADER);
        let fragmentShader = loadShader(fs, GL_FRAGMENT_SHADER);

        match (vertexShader, fragmentShader) {
            (None, None) => (),
            (None, Some(f)) => glDeleteShader(f),
            (Some(v), None) => glDeleteShader(v),
            (Some(v), Some(f)) => {
                glAttachShader(programObject, v);
                glAttachShader(programObject, f);
                glBindAttribLocation(programObject, 0, "vPosition\0".as_ptr() as *const GLchar);
                glLinkProgram(programObject);

                let mut linked = 0;
                glGetProgramiv(programObject, GL_LINK_STATUS, &mut linked);
                if linked == 0 {
                    let mut infoLen = 0;
                    glGetProgramiv(programObject, GL_INFO_LOG_LENGTH, &mut infoLen);
                    if infoLen > 1 {
                        let sptr = allocArray::<u8>(infoLen as usize);
                        glGetProgramInfoLog(programObject, infoLen as GLsizei, core::ptr::null_mut(), sptr as *mut GLchar);
                        libc::puts(sptr as *const i8);
                        freeArray(sptr, infoLen as usize);
                    }

                    glDeleteShader(programObject);
                    return None
                }
            }
        }

        Some(programObject)
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
fn mainLoop(win_: *mut c_void) {
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
fn runMainLoop(win: *mut GLFWwindow) {
    unsafe { emscripten_set_main_loop_arg(mainLoop, win as *mut c_void, 0, 1) };
}

#[cfg(not(target_arch = "wasm32"))]
fn runMainLoop(win: *mut GLFWwindow) {
    unsafe {
        while glfwWindowShouldClose(win) == GLFW_FALSE as c_int && glfwGetKey(win, GLFW_KEY_ESCAPE as c_int) == 0 {
            mainLoop(win as *mut c_void);
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

        let program = loadProgram(&VERTEX_SHADER, &FRAGMENT_SHADER);
        let mut buff = 0;
        glGenBuffers(1, &mut buff);
        let vertices : [f32; 9] =
        [   0.0,    0.5,    0.0,
            -0.5,   -0.5,   0.0,
            0.5,    -0.5,   0.0 ];
        glBindBuffer(GL_ARRAY_BUFFER, buff);
        glBufferData(GL_ARRAY_BUFFER, 4 * 9 as GLsizeiptr, vertices.as_ptr() as *const rs_ctypes::c_void, GL_STATIC_DRAW);

        let state = Box::new(State { program : program, buff : buff});
        glfwSetWindowUserPointer(win, state.asRef() as *const State as *mut ::core::ffi::c_void);
        runMainLoop(win);

        glfwDestroyWindow(win);
        glfwTerminate();
    }
    0
}