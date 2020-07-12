#![no_std]
#![no_main]
use rs_ctypes::*;
use rs_glfw3::bindings::*;
use rs_gles2::bindings::*;

#[cfg(not(test))]
#[panic_handler]
fn alt_std_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
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

        loop {
            let mut width = 0;
            let mut height = 0;
            glfwGetWindowSize(win, &mut width, &mut height);
            glViewport(0, 0, width, height);
            glScissor(0, 0, width, height);
            glClearColor(1.0, 0.0, 0.0, 0.0);
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT | GL_STENCIL_BUFFER_BIT);

            if glfwGetKey(win, GLFW_KEY_ESCAPE as c_int) != 0 {
                break;
            }

            glfwSwapBuffers(win);
            glfwPollEvents();
        }
        glfwDestroyWindow(win);
        glfwTerminate();
    }
    0
}