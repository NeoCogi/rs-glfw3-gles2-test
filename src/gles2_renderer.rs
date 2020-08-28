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
use crate::renderer::*;
use rs_gles2::bindings::*;
use rs_ctypes::*;
use rs_alloc::*;
use rs_math3d::*;

pub struct GLProgram {
    prog_id     : GLuint,
    attribs     : Vec<(VertexAttributeDesc, GLuint)>,
    uniforms    : Vec<(UniformDesc, GLuint)>,
}

impl GLProgram {
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

    pub fn load_program(vs: &str, fs: &str, attribs: &[VertexAttributeDesc], uniforms: &[UniformDesc]) -> Option<Box<dyn Program>> {
        unsafe {
            let program_object = glCreateProgram();
            if program_object == 0 {
                return None
            }

            let vertex_shader    = Self::load_shader(vs, GL_VERTEX_SHADER);
            let fragment_shader  = Self::load_shader(fs, GL_FRAGMENT_SHADER);

            match (vertex_shader, fragment_shader) {
                (None, None) => (),
                (None, Some(f)) => glDeleteShader(f),
                (Some(v), None) => glDeleteShader(v),
                (Some(v), Some(f)) => {
                    glAttachShader(program_object, v);
                    glAttachShader(program_object, f);
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

                        glDeleteProgram(program_object);
                        return None
                    }

                    // done with the shaders
                    glDetachShader(program_object, v);
                    glDetachShader(program_object, f);

                    glDeleteShader(f);
                    glDeleteShader(v);
                }
            }

            let mut prg_attribs = Vec::new();

            for a in attribs {
                let mut s = String::from(a.name().as_str());
                s.push('\0' as u8);

                let au = glGetAttribLocation(program_object, s.as_bytes().as_ptr() as *const GLchar) as GLuint;
                prg_attribs.push((a.clone(), au));
            }

            let mut prg_uniforms = Vec::new();

            for u in uniforms {
                let mut s = String::from(u.name());
                s.push('\0' as u8);

                let au = glGetUniformLocation(program_object, s.as_bytes().as_ptr() as *const GLchar) as GLuint;
                prg_uniforms.push((u.clone(), au));
            }

            let s = Box::new(Self { prog_id: program_object, attribs: prg_attribs, uniforms: prg_uniforms });
            Some(Box::from_raw(Box::into_raw(s) as *mut dyn Program))
        }
    }
}

impl Drop for GLProgram {
    fn drop(&mut self) {
        unsafe { glDeleteProgram(self.prog_id) };
    }
}

impl Program for GLProgram {
    fn attributes(&self) -> &[VertexAttributeDesc] { &[] }
    fn uniforms(&self) -> &[UniformDesc] { &[] }
}

pub struct StaticVertexBuffer {
    buff_id     : GLuint,
    buff_type   : GLenum,
    size        : usize,
    stride      : usize,
}

impl StaticVertexBuffer {
    unsafe fn new_unsafe(stride: usize, buff_data: *const u8, buff_size: usize) -> Self {
        let mut buff = 0;
        glGenBuffers(1, &mut buff);
        glBindBuffer(GL_ARRAY_BUFFER, buff);
        glBufferData(GL_ARRAY_BUFFER, buff_size as GLsizeiptr, buff_data as *const rs_ctypes::c_void, GL_STATIC_DRAW);
        Self {
            stride  : stride,
            buff_id : buff,
            buff_type : GL_STATIC_DRAW,
            size    : buff_size
        }
    }

    pub fn new<T>(data: &[T]) -> Self {
        let s = data.len() * ::core::mem::size_of::<T>();
        unsafe { Self::new_unsafe(::core::mem::size_of::<T>(), data.as_ptr() as *const u8, s) }
    }
}

impl Drop for StaticVertexBuffer {
    fn drop(&mut self) {
        unsafe { glDeleteBuffers(1, &self.buff_id as *const GLuint) }
    }
}

enum IndexType {
    U16,
    U32,
}

pub trait IBData {
    fn index_type() -> IndexType;
}

impl IBData for u16 {
    fn index_type() -> IndexType { IndexType::U16 }
}

impl IBData for u32 {
    fn index_type() -> IndexType { IndexType::U32 }
}


pub struct StaticIndexBuffer {
    buff_id     : GLuint,
    buff_type   : GLenum,
    size        : usize,
    index_type  : IndexType,
}

impl StaticIndexBuffer {
    unsafe fn new_unsafe(index_type: IndexType, buff_data: *const u8, buff_size: usize) -> Self {
        let mut buff = 0;
        glGenBuffers(1, &mut buff);
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, buff);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER, buff_size as GLsizeiptr, buff_data as *const rs_ctypes::c_void, GL_STATIC_DRAW);
        Self {
            index_type : index_type,
            buff_id : buff,
            buff_type : GL_STATIC_DRAW,
            size    : buff_size
        }
    }

    pub fn new<T : IBData>(data: &[T]) -> Self {
        let e_size =
            match T::index_type() {
                IndexType::U16 => 2,
                IndexType::U32 => 4,
            };

        let s = data.len() * e_size;
        unsafe { Self::new_unsafe(T::index_type(), data.as_ptr() as *const u8, s) }
    }
}

impl Drop for StaticIndexBuffer {
    fn drop(&mut self) {
        unsafe { glDeleteBuffers(1, &self.buff_id as *const GLuint) }
    }
}

trait GLUniformBlock {
    fn setup(&self);
}

trait GLVertexBuffer {
    fn buff_id(&self) -> GLuint;
}

trait GLIndexBuffer {
    fn buff_id(&self) -> GLuint;
    fn count(&self) -> GLsizei;
    fn index_type(&self) -> GLenum;
}

impl GLVertexBuffer for StaticVertexBuffer {
    fn buff_id(&self) -> GLuint { self.buff_id }
}

impl GLIndexBuffer for StaticIndexBuffer {
    fn buff_id(&self) -> GLuint { self.buff_id }
    fn count(&self) -> GLsizei {
        match self.index_type {
            IndexType::U16 => (self.size / 2) as GLsizei,
            IndexType::U32 => (self.size / 4) as GLsizei,
        }
    }
    fn index_type(&self) -> GLenum {
        match self.index_type {
            IndexType::U16 => GL_UNSIGNED_SHORT,
            IndexType::U32 => GL_UNSIGNED_INT,
        }
    }
}

trait GLVertexFormat {
    fn gl_elem_count(&self) -> GLuint;
    fn gl_elem_type(&self) -> GLenum;
    fn gl_is_normalized(&self) -> GLboolean;
}

impl GLVertexFormat for VertexFormat {
    fn gl_elem_count(&self) -> GLuint {
        match self {
            VertexFormat::Byte      => 1,
            VertexFormat::Byte2     => 2,
            VertexFormat::Byte3     => 3,
            VertexFormat::Byte4     => 4,
            VertexFormat::SByte     => 1,
            VertexFormat::SByte2    => 2,
            VertexFormat::SByte3    => 3,
            VertexFormat::SByte4    => 4,
            VertexFormat::Int       => 1,
            VertexFormat::Int2      => 2,
            VertexFormat::Int3      => 3,
            VertexFormat::Int4      => 4,
            VertexFormat::Float     => 1,
            VertexFormat::Float2    => 2,
            VertexFormat::Float3    => 3,
            VertexFormat::Float4    => 4,
        }
    }

    fn gl_elem_type(&self) -> GLenum {
        match self {
            VertexFormat::Byte      => GL_UNSIGNED_BYTE,
            VertexFormat::Byte2     => GL_UNSIGNED_BYTE,
            VertexFormat::Byte3     => GL_UNSIGNED_BYTE,
            VertexFormat::Byte4     => GL_UNSIGNED_BYTE,
            VertexFormat::SByte     => GL_BYTE,
            VertexFormat::SByte2    => GL_BYTE,
            VertexFormat::SByte3    => GL_BYTE,
            VertexFormat::SByte4    => GL_BYTE,
            VertexFormat::Int       => GL_INT,
            VertexFormat::Int2      => GL_INT,
            VertexFormat::Int3      => GL_INT,
            VertexFormat::Int4      => GL_INT,
            VertexFormat::Float     => GL_FLOAT,
            VertexFormat::Float2    => GL_FLOAT,
            VertexFormat::Float3    => GL_FLOAT,
            VertexFormat::Float4    => GL_FLOAT,
        }
    }

    fn gl_is_normalized(&self) -> GLboolean {
        let r = match self {
            VertexFormat::Byte      => true,
            VertexFormat::Byte2     => true,
            VertexFormat::Byte3     => true,
            VertexFormat::Byte4     => true,
            VertexFormat::SByte     => true,
            VertexFormat::SByte2    => true,
            VertexFormat::SByte3    => true,
            VertexFormat::SByte4    => true,
            VertexFormat::Int       => false,
            VertexFormat::Int2      => false,
            VertexFormat::Int3      => false,
            VertexFormat::Int4      => false,
            VertexFormat::Float     => false,
            VertexFormat::Float2    => false,
            VertexFormat::Float3    => false,
            VertexFormat::Float4    => false,
        };
        r as GLboolean
    }
}

fn uniform_ptr_to_slice<'a, T>(ptr: *const c_void, offset: usize, count: usize) -> &'a [T] {
    let cptr = ptr as *const u8;
    let _cptr = unsafe { cptr.offset(offset as isize) };
    let tptr = _cptr as *const T;
    unsafe { core::slice::from_raw_parts(tptr, count) }
}

fn setup_uniforms(uniforms: *const c_void, data_desc_layout: &[UniformDataDesc], prg_desc_layout: &[(UniformDesc, GLuint)]) {
    unsafe {
        for i in 0..data_desc_layout.len() {
            let offset = data_desc_layout[i].offset();
            let location = prg_desc_layout[i].1 as GLint;
            match &data_desc_layout[i].desc().format() {
                UniformDataType::Int  => { let s : &[i32]     = uniform_ptr_to_slice(uniforms, offset, 1);  glUniform1iv(location, 1, s.as_ptr()); },
                UniformDataType::Int2 => { let s : &[i32]     = uniform_ptr_to_slice(uniforms, offset, 2);  glUniform2iv(location, 1, s.as_ptr()); },
                UniformDataType::Int3 => { let s : &[i32]     = uniform_ptr_to_slice(uniforms, offset, 3);  glUniform3iv(location, 1, s.as_ptr()); },
                UniformDataType::Int4 => { let s : &[i32]     = uniform_ptr_to_slice(uniforms, offset, 4);  glUniform4iv(location, 1, s.as_ptr()); },
                UniformDataType::Float  => { let s : &[f32]   = uniform_ptr_to_slice(uniforms, offset, 1);  glUniform1fv(location, 1, s.as_ptr()); },
                UniformDataType::Float2 => { let s : &[f32]   = uniform_ptr_to_slice(uniforms, offset, 2);  glUniform2fv(location, 1, s.as_ptr()); },
                UniformDataType::Float3 => { let s : &[f32]   = uniform_ptr_to_slice(uniforms, offset, 3);  glUniform3fv(location, 1, s.as_ptr()); },
                UniformDataType::Float4 => { let s : &[f32]   = uniform_ptr_to_slice(uniforms, offset, 4);  glUniform4fv(location, 1, s.as_ptr()); },
                UniformDataType::Float2x2 => { let s : &[f32] = uniform_ptr_to_slice(uniforms, offset, 4);  glUniformMatrix2fv(location, 1, false as GLboolean, s.as_ptr()); },
                UniformDataType::Float3x3 => { let s : &[f32] = uniform_ptr_to_slice(uniforms, offset, 9);  glUniformMatrix3fv(location, 1, false as GLboolean, s.as_ptr()); },
                UniformDataType::Float4x4 => { let s : &[f32] = uniform_ptr_to_slice(uniforms, offset, 16); glUniformMatrix4fv(location, 1, false as GLboolean, s.as_ptr()); },
            }
        }
    }
}


fn draw_raw(prg: &GLProgram, buff: &StaticVertexBuffer, uniforms: *const c_void, data_desc_layout: &[UniformDataDesc]) {
    unsafe {
        glUseProgram(prg.prog_id);
        glBindBuffer(GL_ARRAY_BUFFER, buff.buff_id());
        for (a, l) in prg.attribs.iter() {
            glVertexAttribPointer(*l, a.format().gl_elem_count() as GLint, a.format().gl_elem_type(), a.format().gl_is_normalized(), buff.stride as GLint, a.offset() as *const c_void);
            glEnableVertexAttribArray(*l);
        }

        setup_uniforms(uniforms, data_desc_layout, prg.uniforms.as_slice());
        glDrawArrays(GL_TRIANGLES, 0, (buff.size / buff.stride) as GLint);

        for (_, l) in prg.attribs.iter() {
            glDisableVertexAttribArray(*l);
        }
    }
}

pub fn draw<T: UniformBlock>(prg: &Box<dyn Program>, buff: &StaticVertexBuffer, uniforms: &T) {
    let gl_prog = unsafe { &*(prg.as_ref() as *const dyn Program as *const GLProgram) };
    let u_ptr   = uniforms as *const T as *const c_void;
    draw_raw(gl_prog, buff, u_ptr, T::descriptors().as_slice());
}

fn draw_indexed_raw(prg: &GLProgram, vb: &StaticVertexBuffer, ib: &StaticIndexBuffer, uniforms: *const c_void, data_desc_layout: &[UniformDataDesc]) {
    unsafe {
        glUseProgram(prg.prog_id);
        glBindBuffer(GL_ARRAY_BUFFER, vb.buff_id());
        for (a, l) in prg.attribs.iter() {
            glVertexAttribPointer(*l, a.format().gl_elem_count() as GLint, a.format().gl_elem_type(), a.format().gl_is_normalized(), vb.stride as GLint, a.offset() as *const c_void);
            glEnableVertexAttribArray(*l);
        }

        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ib.buff_id);

        setup_uniforms(uniforms, data_desc_layout, prg.uniforms.as_slice());
        glDrawElements(GL_TRIANGLES, ib.count(), ib.index_type(), core::ptr::null() as *const rs_ctypes::c_void);

        for (_, l) in prg.attribs.iter() {
            glDisableVertexAttribArray(*l);
        }
    }
}

pub fn draw_indexed<T: UniformBlock>(prg: &Box<dyn Program>, vb: &StaticVertexBuffer, ib: &StaticIndexBuffer, uniforms: &T) {
    let gl_prog = unsafe { &*(prg.as_ref() as *const dyn Program as *const GLProgram) };
    let u_ptr   = uniforms as *const T as *const c_void;
    draw_indexed_raw(gl_prog, vb, ib, u_ptr, T::descriptors().as_slice());
}