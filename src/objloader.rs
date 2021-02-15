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
use core::result::Result;
use rs_alloc::*;
use rs_streams::*;

use rs_math3d::*;

pub struct IdTri {
    verts   : [u32; 3],
    uvs     : [u32; 3]
}

impl IdTri {
    pub fn new(verts: [u32; 3], uvs: [u32; 3]) -> Self { Self { verts: verts, uvs: uvs } }
}


pub struct IdQuad {
    verts   : [u32; 4],
    uvs     : [u32; 4]
}

impl IdQuad {
    pub fn new(verts: [u32; 4], uvs: [u32; 4]) -> Self { Self { verts: verts, uvs: uvs } }
}


fn parse_vec3(parts: &[&str], verts: &mut Vec<Vec3f>) -> Result<i32, String> {
    if parts.len() != 3 {
        return Err(String::from("expecting 3 floats"))
    }

    let f0 : Result<f32, _> = parts[0].parse();
    let f1 : Result<f32, _> = parts[1].parse();
    let f2 : Result<f32, _> = parts[2].parse();

    match (f0, f1, f2) {
        (Ok(f0), Ok(f1), Ok(f2)) => {
            verts.push(Vec3f::new(f0, f1, f2));
            Ok(0)
        },
        _ => Err(String::from("float parse error"))
    }
}

fn parse_vec2(parts: &[&str], uvws: &mut Vec<Vec3f>) -> Result<i32, String> {
    if parts.len() != 2 {
        return Err(String::from("expecting 2 floats"))
    }

    let f0 : Result<f32, _> = parts[0].parse();
    let f1 : Result<f32, _> = parts[1].parse();

    match (f0, f1) {
        (Ok(f0), Ok(f1)) => {
            uvws.push(Vec3f::new(f0, f1, 0.0));
            Ok(0)
        },
        _ => Err(String::from("float parse error"))
    }
}

fn parse_part(part: &str) -> Result<(u32, u32), String> {
    let parts : Vec<&str> = part.split('/').collect();
    if parts.len() != 1 && parts.len() != 3 {
        return Result::Err(String::from("expecting vertex or vertex//uv"))
    }

    let v : Result<u32, _> = parts[0].parse();
    let uv : Result<u32, _> = if parts.len() == 1 { Ok(0) } else { parts[2].parse() };

    match (v, uv) {
        (Ok(v), Ok(uv)) => Result::Ok((v, uv)),
        _ => Result::Err(String::from("expecting vertex//uv or vertex"))
    }
}

fn parse_face(parts: &[&str], tris: &mut Vec<IdTri>, quads: &mut Vec<IdQuad>) -> Result<i32, String> {
    if parts.len() != 3 && parts.len() != 4 {
        return Result::Err(String::from("expecting a triangle or quad"))
    }

    let p0 = parse_part(&parts[0]);
    let p1 = parse_part(&parts[1]);
    let p2 = parse_part(&parts[2]);

    if parts.len() == 3 {
        match (&p0, &p1, &p2) {
            (Ok((v0, uv0)), Ok((v1, uv1)), Ok((v2, uv2))) => {
                tris.push(IdTri::new([*v0 - 1, *v1 - 1, *v2 - 1], [*uv0 - 1, *uv1 - 1, *uv2 - 1]));
                return Ok(0)
            }
            _ => { return Err(String::new()) }
        }
    }

    let p3 = parse_part(&parts[3]);
    match (&p0, &p1, &p2, &p3) {
        (Ok((v0, uv0)), Ok((v1, uv1)), Ok((v2, uv2)), Ok((v3, uv3))) => { quads.push(IdQuad::new([*v0 - 1, *v1 - 1, *v2 - 1, *v3 - 1], [*uv0 - 1, *uv1 - 1, *uv2 - 1, *uv3 - 1])); }
        _ => { return Err(String::new()) }
    }

    Ok(0)
}

fn parse_line(line: &str, verts: &mut Vec<Vec3f>, uvws: &mut Vec<Vec3f>, tris: &mut Vec<IdTri>, quads: &mut Vec<IdQuad>) -> Result<i32, String> {
    if line == "" {
        return Result::Ok(0)
    }

    let parts : Vec<&str> = line.split(|x| x == ' ' || x == '\t').filter(|&x| x != "").collect();
    if parts.len() == 0 {
        return Result::Err(String::from("No part!"))
    }

    if parts[0].starts_with('#') {
        return Result::Ok(1)
    }

    match parts[0] {
        "v"     => parse_vec3(&parts[1..], verts),
        "vt"    => parse_vec2(&parts[1..], uvws),
        "f"     => parse_face(&parts[1..], tris, quads),
        _       => Result::Ok(2)
    }
}

pub struct Mesh {
    verts   : Vec<Vec3f>,
    uvws    : Vec<Vec3f>,
    tris    : Vec<IdTri>,
    quads   : Vec<IdQuad>
}

impl Mesh {

    pub fn verts(&self) -> &Vec<Vec3f>  { &self.verts }
    pub fn uvws(&self)  -> &Vec<Vec3f>  { &self.uvws }
    pub fn tris(&self)  -> &Vec<IdTri>  { &self.tris }
    pub fn quads(&self) -> &Vec<IdQuad> { &self.quads }


    pub fn from(verts: Vec<Vec3f>, uvws: Vec<Vec3f>, tris: Vec<IdTri>, quads: Vec<IdQuad>) -> Self {
        Self { verts: verts, uvws: uvws, tris: tris, quads: quads }
    }


    pub fn read_obj(path: &str) -> Result<Mesh, String> {
        let file = File::open(path);
        match file {
            Ok(_) => (),
            Err(_) => return Err(String::from("Could not open file"))
        }

        let mut f = file.unwrap();
        let mut lines = String::new();
        f.read_to_string(&mut lines).unwrap();

        let mut verts = Vec::<Vec3f>::new();
        let mut uvws = Vec::<Vec3f>::new();
        let mut tris = Vec::<IdTri>::new();
        let mut quads = Vec::<IdQuad>::new();

        for l in lines.lines() {
            match parse_line(l.as_str(), &mut verts, &mut uvws, &mut tris, &mut quads) {
                Ok(_) => (),
                Err(err) => return Err(err)
            }
        }

        Ok(Mesh::from(verts, uvws, tris, quads))
    }
}

#[repr(C)]
pub struct GPUVertex {
    pub pos     : Vec3f,
    pub normal  : Vec3f,
    pub uv      : Vec2f,
}

pub struct GPUMesh {
    verts   : Vec<GPUVertex>,
    tris    : Vec<u32>,
}

impl GPUMesh {
    pub fn from(mesh: &Mesh) -> Self {
        let mut gpv = Vec::new();
        let mut tris = Vec::new();

        for t in mesh.tris().iter() {
            let v0 = mesh.verts[t.verts[0] as usize];
            let v1 = mesh.verts[t.verts[1] as usize];
            let v2 = mesh.verts[t.verts[2] as usize];

            let n = rs_math3d::tri_normal(&v0, &v1, &v2);

            let uv0 = mesh.verts[t.uvs[0] as usize];
            let uv1 = mesh.verts[t.uvs[1] as usize];
            let uv2 = mesh.verts[t.uvs[2] as usize];

            let idx = gpv.len();
            gpv.push(GPUVertex { pos: v0, normal: n.clone(), uv: Vec2f::new(uv0.x, uv0.y) });
            gpv.push(GPUVertex { pos: v1, normal: n.clone(), uv: Vec2f::new(uv1.x, uv1.y) });
            gpv.push(GPUVertex { pos: v2, normal: n.clone(), uv: Vec2f::new(uv2.x, uv2.y) });

            tris.push(idx as u32);
            tris.push(idx as u32 + 1);
            tris.push(idx as u32 + 2);
        }

        for q in mesh.quads().iter() {
            let v0 = mesh.verts[q.verts[0] as usize];
            let v1 = mesh.verts[q.verts[1] as usize];
            let v2 = mesh.verts[q.verts[2] as usize];
            let v3 = mesh.verts[q.verts[3] as usize];

            let n = rs_math3d::quad_normal(&v0, &v1, &v2, &v3);

            let uv0 = mesh.verts[q.uvs[0] as usize];
            let uv1 = mesh.verts[q.uvs[1] as usize];
            let uv2 = mesh.verts[q.uvs[2] as usize];
            let uv3 = mesh.verts[q.uvs[3] as usize];

            let idx = gpv.len();
            gpv.push(GPUVertex { pos: v0, normal: n.clone(), uv: Vec2f::new(uv0.x, uv0.y) });
            gpv.push(GPUVertex { pos: v1, normal: n.clone(), uv: Vec2f::new(uv1.x, uv1.y) });
            gpv.push(GPUVertex { pos: v2, normal: n.clone(), uv: Vec2f::new(uv2.x, uv2.y) });
            gpv.push(GPUVertex { pos: v3, normal: n.clone(), uv: Vec2f::new(uv3.x, uv3.y) });

            tris.push(idx as u32);
            tris.push(idx as u32 + 1);
            tris.push(idx as u32 + 2);

            tris.push(idx as u32 + 2);
            tris.push(idx as u32 + 3);
            tris.push(idx as u32);
        }

        Self { verts: gpv, tris: tris }
    }

    pub fn verts(&self) -> &[GPUVertex] { self.verts.as_slice() }
    pub fn tris(&self) -> &[u32] { self.tris.as_slice() }
}