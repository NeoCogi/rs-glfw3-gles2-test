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
use rs_ctypes::*;
use rs_alloc::*;
use rs_math3d::*;


#[derive(Clone)]
pub enum VertexFormat {
    Byte,
    Byte2,
    Byte3,
    Byte4,
    SByte,
    SByte2,
    SByte3,
    SByte4,
    Int,
    Int2,
    Int3,
    Int4,
    Float,
    Float2,
    Float3,
    Float4,
}


#[derive(Clone)]
pub struct VertexAttributeDesc {
    name        : String,
    format      : VertexFormat,
    offset      : usize,
}

impl VertexAttributeDesc {
    pub fn new(name: String, format: VertexFormat, offset: usize) -> Self { Self { name: name, format: format, offset: offset } }
    pub fn name(&self)      -> &String  { &self.name  }
    pub fn format(&self)    -> VertexFormat   { self.format.clone() }
    pub fn offset(&self)    -> usize    { self.offset }
}

#[derive(Clone)]
pub enum UniformDataType {
    Int,
    Int2,
    Int3,
    Int4,
    Float,
    Float2,
    Float3,
    Float4,
    Float2x2,
    Float3x3,
    Float4x4,
}

#[derive(Clone)]
pub struct UniformDesc {
    name        : String,
    format      : UniformDataType,
    count       : usize,
}

impl UniformDesc {
    pub fn new(name: String, format: UniformDataType, count: usize) -> Self { Self { name: name, format: format, count: count } }
    pub fn name(&self)      -> &str     { self.name.as_str() }
    pub fn format(&self)    -> UniformDataType { self.format.clone() }
    pub fn count(&self)     -> usize    { self.count }
}

#[derive(Clone)]
pub struct UniformDataDesc {
    desc        : UniformDesc,
    offset      : usize,
}

impl UniformDataDesc {
    pub fn new(name: String, format: UniformDataType, count: usize, offset: usize) -> Self { Self { desc: UniformDesc::new(name, format, count), offset: offset } }
    pub fn offset(&self)    -> usize    { self.offset }
    pub fn desc(&self)      -> &UniformDesc   { &self.desc }
}

pub trait UniformBlock {
    fn descriptors() -> Vec<UniformDataDesc>;
}

////////////////////////////////////////////////////////////////////////////////
/// PipelineDesc
////////////////////////////////////////////////////////////////////////////////

pub enum IndexType {
    UInt16,
    UInt32,
}

pub enum ImageType {
    IT2D,
    ITCube,
    IT3D,
    ITArray,
}

pub enum SamplerType {
    Float,
    SInt,
    UInt,
}

pub enum PrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
}

pub enum Filter {
    Nearest,
    Linear,
    NearestMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapNearest,
    LinearMipmapLinear,
}

pub enum Wrap {
    Repeat,
    ClampToEdge,
    ClampToBorder,
    MirroredRepeat,
}

pub enum CullMode {
    CCW,
    CW,
}

pub trait Program {
    fn attributes(&self) -> &[VertexAttributeDesc];
    fn uniforms(&self) -> &[UniformDesc];
}
