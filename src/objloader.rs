use core::result::Result;
use rs_collections::*;
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
