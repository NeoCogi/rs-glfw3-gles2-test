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

        loop {
            let mut width = 0;
            let mut height = 0;
            glfwGetWindowSize(win, &mut width, &mut height);
            glViewport(0, 0, width, height);
            glScissor(0, 0, width, height);
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);

            if glfwGetKey(win, GLFW_KEY_ESCAPE as c_int) != 0 {
                break;
            }

            match program {
                Some(p) => {
                    let vertices : [f32; 9]= [ 0.0, 0.5, 0.0,
                                            -0.5, -0.5, 0.0,
                                            0.5, 0.5, 0.0];
                    glUseProgram(p);
                    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE as u8, 0, vertices.as_ptr() as *const c_void);
                    glEnableVertexAttribArray(0);
                    glDrawArrays(GL_TRIANGLES, 0, 3);
                },
                None => ()
            }

            glfwSwapBuffers(win);
            glfwPollEvents();
        }
        glfwDestroyWindow(win);
        glfwTerminate();
    }
    0
}