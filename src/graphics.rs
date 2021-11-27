
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use cgmath::Matrix4;
use std::collections::HashMap;
use std::ffi::c_void;
use std::path::PathBuf;
use gl::types::*;
use image::{DynamicImage, GenericImageView, RgbaImage};

#[derive(PartialEq, EnumIter, Clone, Copy)]
pub enum Usage {
    POSITIONS,
    COLORS,
    NORMALS,
    TEXCOORDS,
    INDICES,
}
impl Usage {
    pub fn position(&self) -> u8 {
        match *self {
            Usage::POSITIONS => 0,
            Usage::COLORS => 1,
            Usage::NORMALS => 2,
            Usage::TEXCOORDS => 3,
            Usage::INDICES => 4,
        }
    }
    
    pub fn offset(&self) -> u8 {
        match *self {
            Usage::POSITIONS => 3,
            Usage::COLORS => 4,
            Usage::NORMALS => 3,
            Usage::TEXCOORDS => 2,
            Usage::INDICES => 3,
        }
    }
}

#[derive(Clone, Copy)]
pub struct VertexAttributes {
    pub vertex_size: u8,
    has_positions: bool,
    has_colors: bool,
    has_normals: bool,
    has_tex_coords: bool,
}
impl VertexAttributes {
    pub fn with(has_positions: bool, has_colors: bool, has_normals: bool, has_tex_coords: bool) -> VertexAttributes {
        let mut va = VertexAttributes { vertex_size: 0, has_positions, has_colors, has_normals, has_tex_coords };
        if has_positions {
            va.vertex_size += Usage::POSITIONS.offset();
        }
        if has_colors {
            va.vertex_size += Usage::COLORS.offset();
        }
        if has_normals {
            va.vertex_size += Usage::NORMALS.offset();
        }
        if has_tex_coords {
            va.vertex_size += Usage::TEXCOORDS.offset();
        }
        
        va
    }
    
    pub fn offset(&self, usage: Usage) -> u8 {
        let mut off = 0;
        
        if self.has_positions {
            if usage == Usage::POSITIONS {
                return off;
            }
            off += Usage::POSITIONS.offset();
        }
        if self.has_colors {
            if usage == Usage::COLORS {
                return off;
            }
            off += Usage::COLORS.offset();
        }
        if self.has_normals {
            if usage == Usage::NORMALS {
                return off;
            }
            off += Usage::NORMALS.offset();
        }
        if self.has_tex_coords {
            if usage == Usage::TEXCOORDS {
                return off;
            }
        }
        
        0
    }
    
    pub fn usage(&self, usage: Usage) -> bool {
        (usage == Usage::POSITIONS && self.has_positions)
            | (usage == Usage::COLORS && self.has_colors)
            | (usage == Usage::NORMALS && self.has_normals)
            | (usage == Usage::TEXCOORDS && self.has_tex_coords)
    }
}

/////////////////////

pub struct VertexBufferObject {
    vbo_index: GLuint,
    name: GLuint,
    usage: Usage,
    data: Vec<f32>,
    offset: usize,
    dirty: bool,
}
impl VertexBufferObject {
    pub fn new(usage: Usage) -> VertexBufferObject {
        let vbo_index = usage.position().into();
        
        let mut name: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut name);
            
            if usage == Usage::INDICES {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, name);
                gl::EnableVertexAttribArray(vbo_index);
                gl::VertexAttribPointer(vbo_index, usage.offset().into(), gl::INT, gl::FALSE, 0, std::ptr::null());
            } else {
                gl::BindBuffer(gl::ARRAY_BUFFER, name);
                gl::EnableVertexAttribArray(vbo_index);
                gl::VertexAttribPointer(vbo_index, usage.offset().into(), gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            }
        }
        
        VertexBufferObject {
            vbo_index,
            name: name,
            usage: usage,
            data: vec![0.0; 0],
            offset: 0,
            dirty: false,
        }
    }
    
    pub fn bind(&mut self) {
        unsafe {
            if self.usage == Usage::INDICES {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.name);
            } else {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.name);
            }
            
            if self.dirty {
                if self.usage == Usage::INDICES {
                    let intdata = Self::data_ints(&self.data);
                    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (intdata.len() * 4) as isize, intdata.as_ptr() as *const GLvoid, gl::DYNAMIC_DRAW);
                } else {
                    gl::BufferData(gl::ARRAY_BUFFER, (self.data.len() * 4) as isize, self.data.as_ptr() as *const GLvoid, gl::DYNAMIC_DRAW);
                    gl::VertexAttribPointer(self.vbo_index, self.usage.offset().into(), gl::FLOAT, gl::FALSE, 0, std::ptr::null());
                }
            }
            
            self.dirty = false;
        }
    }
    
    pub fn set_data(&mut self, data: &[f32]) {
        self.data.clear();
        self.data.extend_from_slice(data);
        self.offset = self.data.len();
        self.dirty = true;
    }
    
    pub fn add_data_slice(&mut self, data: &[f32]) {
        self.data.extend_from_slice(&data);
        self.offset += data.len();
        self.dirty = true;
    }
    
    pub fn add_data(&mut self, data: f32) {
        self.data.push(data);
        self.offset += 1;
        self.dirty = true;
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
        self.offset = 0;
        self.dirty = true;
    }
    
    pub fn dispose(&self) {
        unsafe {
            gl::DeleteBuffers(1, [self.name].as_ptr());
        }
    }
    
    fn data_ints(data: &Vec<f32>) -> Vec<i32> {
        let mut d = vec![0i32; data.len()];
        for val in data.iter() {
            d.push(*val as i32);
        }
        
        d
    }
}

pub struct VertexArrayObject {
    name: GLuint,
    vbos: Vec<VertexBufferObject>,
    vbo_indices: VertexBufferObject,
    bound: bool,
}
impl VertexArrayObject {
    pub fn new(attribs: VertexAttributes) -> VertexArrayObject {
        let mut name: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut name);
            gl::BindVertexArray(name);
        }
        
        let mut vbos = Vec::new();
        for usage in Usage::iter() {
            if attribs.usage(usage) {
                vbos.push(VertexBufferObject::new(usage));
            }
        }
        
        VertexArrayObject {
            name: name,
            vbos: vbos,
            vbo_indices: VertexBufferObject::new(Usage::INDICES),
            bound: false
        }
    }
    
    pub fn vertex(&mut self, data: &[f32]){
        let mut offset = 0;
        for vbo in &mut self.vbos {
            for i in 0..vbo.usage.offset() {
                vbo.add_data(data[(offset + i) as usize]);
            }
            offset += vbo.usage.offset();
        }
    }
    
    //TODO pub fn indices( slice ) { vboIndices.addData( slice ) }
    
    pub fn clear(&mut self) {
        for vbo in &mut self.vbos {
            vbo.clear();
        }
        self.vbo_indices.clear();
    }
    
    pub fn get_vertex_offset(&self, usage: Usage) -> u8 {
        let mut i = 0;
        for vbo in &self.vbos {
            if usage == vbo.usage {
                return i;
            }
            i += vbo.usage.offset();
        }
        
        i
    }
    
    pub fn bind(&mut self) {
        unsafe {
            gl::BindVertexArray(self.name);
        }
        
        for vbo in &mut self.vbos {
            vbo.bind();
        }
        self.vbo_indices.bind();
        
        self.bound = true;
    }
    
    pub fn render(&self, primitive: GLenum) {
        if !self.bound {
            panic!("VertexArrayObject must be bound before rendering!");
        }
        
        unsafe {
            if self.vbo_indices.offset > 0 {
                gl::DrawElements(primitive, self.vbo_indices.offset as GLint, gl::UNSIGNED_INT, std::ptr::null());
            } else {
                gl::DrawArrays(primitive, 0, self.vbos[0].data.len() as GLint);
            }
        }
    }
    
    pub fn unbind(&mut self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        
        self.bound = false;
    }
    
    pub fn dispose(&self) {
        for vbo in &self.vbos {
            vbo.dispose();
        }
        self.vbo_indices.dispose();
        
        unsafe {
            gl::DeleteVertexArrays(1, [self.name].as_ptr());
        }
    }
}

pub struct ShaderProgram {
    program_id: GLuint,
    vertex_shader_id: GLuint,
    fragment_shader_id: GLuint,
    uniforms: HashMap<String, GLint>,
    pub linked: bool,
}
impl ShaderProgram {
    pub fn new() -> ShaderProgram {
        unsafe {
            ShaderProgram {
                program_id: gl::CreateProgram(),
                vertex_shader_id: 0,
                fragment_shader_id: 0,
                uniforms: HashMap::new(),
                linked: false
            }
        }
    }
    
    pub fn create_vertex_shader(&mut self, code: &str) {
        self.vertex_shader_id = Self::create_shader(code, gl::VERTEX_SHADER, self.program_id);
    }
    
    pub fn create_fragment_shader(&mut self, code: &str) {
        self.fragment_shader_id = Self::create_shader(code, gl::FRAGMENT_SHADER, self.program_id);
    }
    
    fn create_shader(code: &str, shader_type: GLenum, program_id: GLuint) -> GLuint {
        unsafe {
            let id = gl::CreateShader(shader_type);
            if id == 0 {
                panic!("Error creating shader. Type {:?}", shader_type);
            }
            
            let ptr: *const u8 = code.as_bytes().as_ptr();
            let ptr_i8: *const i8 = std::mem::transmute(ptr);
            gl::ShaderSource(id, 1, &ptr_i8, &(code.len() as GLint));
            gl::CompileShader(id);
            
            if Self::getsiv(id, gl::COMPILE_STATUS) == 0 {
                panic!("Error compiling shader code: {}", Self::getslog(id));
            }
            
            gl::AttachShader(program_id, id);
            
            id
        }
    }
    
    pub fn link(&mut self) {
        unsafe {
            gl::LinkProgram(self.program_id);
            if Self::getpiv(self.program_id, gl::LINK_STATUS) == 0 {
                panic!("Error linking shader code: {}", Self::getplog(self.program_id));
            }
            
            if self.vertex_shader_id != 0 {
                gl::DetachShader(self.program_id, self.vertex_shader_id);
            }
            
            if self.fragment_shader_id != 0 {
                gl::DetachShader(self.program_id, self.fragment_shader_id);
            }
            
            gl::ValidateProgram(self.program_id);
            if Self::getpiv(self.program_id, gl::VALIDATE_STATUS) == 0 {
                println!("Warning validating shader code: {}", Self::getplog(self.program_id));
            }
            
            self.linked = true;
        }
    }
    
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }
    
    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
    
    pub fn set_uniform_mat4f(&self, name: &str, val: Matrix4<f32>) {
        unsafe {
            let mat: &[[f32; 4]; 4] = val.as_ref();
            let ptr: *const f32 = std::mem::transmute(mat);
            gl::UniformMatrix4fv(Self::check_uniform(self, name), 1, gl::FALSE, ptr);
        }
    }
    
    pub fn set_uniform1f32(&self, name: &str, val: f32) {
        unsafe {
            gl::Uniform1f(Self::check_uniform(self, name), val);
        }
    }
    
    pub fn set_uniform1i32(&self, name: &str, val: i32) {
        unsafe {
            gl::Uniform1i(Self::check_uniform(self, name), val);
        }
    }
    
    fn check_uniform(&self, name: &str) -> GLint {
        if self.uniforms.contains_key(name) { // return existing uniform location
            return *self.uniforms.get(name).unwrap();
        } else { // else create a new one, store it, and return
            let loc;
            let c_name = std::ffi::CString::new(name).unwrap();
            unsafe {
                loc = gl::GetUniformLocation(self.program_id, c_name.as_ptr());
            }
            
            loc
        }
    }
    
    fn getsiv(shader_id: GLuint, param: GLenum) -> GLint { // GetShaderiv
        let mut val = 0;
        unsafe {
            gl::GetShaderiv(shader_id, param, &mut val);
        }
        
        val
    }
    
    fn getslog(shader_id: GLuint) -> String { // GetShaderInfoLog
        let len = Self::getsiv(shader_id, gl::INFO_LOG_LENGTH);
        
        let mut buf = Vec::with_capacity(len as usize);
        let buf_ptr = buf.as_mut_ptr() as *mut GLchar;
        unsafe {
            gl::GetShaderInfoLog(shader_id, len, std::ptr::null_mut(), buf_ptr);
            buf.set_len(len as usize);
        }
        
        String::from_utf8(buf).unwrap()
    }
    
    fn getpiv(program_id: GLuint, param: GLenum) -> GLint { // GetShaderiv
        let mut val = 0;
        unsafe {
            gl::GetProgramiv(program_id, param, &mut val);
        }
        
        val
    }
    
    fn getplog(program_id: GLuint) -> String { // GetShaderInfoLog
        let len = Self::getpiv(program_id, gl::INFO_LOG_LENGTH);
        
        let mut buf = Vec::with_capacity(len as usize);
        let buf_ptr = buf.as_mut_ptr() as *mut GLchar;
        unsafe {
            gl::GetProgramInfoLog(program_id, len, std::ptr::null_mut(), buf_ptr);
            buf.set_len(len as usize);
        }
        
        String::from_utf8(buf).unwrap()
    }
}

pub struct Mesh {
    vao: VertexArrayObject,
    attribs: VertexAttributes,
}
impl Mesh {
    pub fn new(attribs: VertexAttributes) -> Self {
        Self {
            vao: VertexArrayObject::new(attribs),
            attribs: attribs,
        }
    }
    
    pub fn vertex(&mut self, data: &[f32]) {
        self.vao.vertex(data);
    }
    
    pub fn clear(&mut self) {
        self.vao.clear();
    }
    
    pub fn render(&mut self, shader: &ShaderProgram, bind_externally: bool, primitive: GLenum, proj_model_view: Matrix4<f32>) {
        if !bind_externally {
            shader.bind();
        }
        
        self.vao.bind();
        shader.set_uniform_mat4f("projModelView", proj_model_view);
        self.vao.render(primitive);
        self.vao.unbind();
        
        if !bind_externally {
            shader.unbind();
        }
    }
    
    pub fn get_vertex_offset(&self, usage: Usage) -> u8 {
        self.vao.get_vertex_offset(usage)
    }
}

pub struct MeshRenderer {
    shader: ShaderProgram,
    mesh: Mesh,
    next_vertex: Vec<f32>,
}
impl MeshRenderer {
    pub fn new(vertex_shader_code: &str, fragment_shader_code: &str) -> Self {
        let mut shader = ShaderProgram::new();
        shader.create_vertex_shader(vertex_shader_code);
        shader.create_fragment_shader(fragment_shader_code);
        shader.link();
        
        let mesh = Mesh::new(VertexAttributes::with(true, true, false, false));
        let next = vec![0f32; mesh.attribs.vertex_size.into()];
        Self {
            shader: shader,
            mesh: mesh,
            next_vertex: next,
        }
    }
    
    pub fn render(&mut self, combined: Matrix4<f32>, primitive: GLenum) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            
            self.mesh.render(&self.shader, false, primitive, combined);
            
            gl::Disable(gl::BLEND);
        }
    }
    
    pub fn color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        if self.mesh.attribs.has_colors {
            let offset = self.mesh.get_vertex_offset(Usage::COLORS) as usize;
            self.next_vertex[offset] = r;
            self.next_vertex[offset+1] = g;
            self.next_vertex[offset+2] = b;
            self.next_vertex[offset+3] = a;
        }
    }
    
    pub fn normal(&mut self, x: f32, y: f32, z: f32) {
        if self.mesh.attribs.has_normals {
            let offset = self.mesh.get_vertex_offset(Usage::NORMALS) as usize;
            self.next_vertex[offset] = x;
            self.next_vertex[offset+1] = y;
            self.next_vertex[offset+2] = z;
        }
    }
    
    pub fn tex_coord(&mut self, u: f32, v: f32) {
        if self.mesh.attribs.has_tex_coords {
            let offset = self.mesh.get_vertex_offset(Usage::TEXCOORDS) as usize;
            self.next_vertex[offset] = u;
            self.next_vertex[offset+1] = v;
        }
    }
    
    pub fn vertex(&mut self, x: f32, y: f32, z: f32) {
        self.next_vertex[0] = x;
        self.next_vertex[1] = y;
        self.next_vertex[2] = z;
        self.mesh.vertex(&self.next_vertex);
        
        self.next_vertex.fill(0.0);
    }
    
    pub fn clear(&mut self) {
        self.mesh.clear();
        self.next_vertex.fill(0.0);
    }
}


pub struct Texture {
    id: u32,
    width: u32,
    height: u32,
    original_image: RgbaImage,
}
impl Texture {
    pub fn from_path(path: &PathBuf) -> Self {
        let img = image::open(path).unwrap().into_rgba8();
        
        Self::from_image(img)
    }
    
    pub fn from_image(mut img: RgbaImage) -> Self {
        img = image::imageops::flip_vertical(&img);
        
        let width = img.width();
        let height = img.height();
        let original_image = img.clone();
        
        Self {
            id: Self::gl_gen(img),
            width,
            height,
            original_image,
        }
    }
    
    fn gl_gen(img: RgbaImage) -> u32 {
        let mut id = 0;
        
        unsafe {
            gl::GenTextures(1, &mut id);
            
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, img.width() as GLsizei, img.height() as GLsizei, 0, gl::RGBA, gl::UNSIGNED_BYTE, img.into_raw().as_ptr() as *const c_void);
            
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        
        id
    }
    
    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
    
    /// Returns a clone of this image, with every pixel multiplied by the provided color
    pub fn multiply(&self, r: f32, g: f32, b: f32, a: f32) -> Self {
        let mut img = self.original_image.clone();
        img.pixels_mut().for_each(|pixel| {
            pixel.0[0] = ((pixel.0[0] as f32) * r) as u8;
            pixel.0[1] = ((pixel.0[1] as f32) * g) as u8;
            pixel.0[2] = ((pixel.0[2] as f32) * b) as u8;
            pixel.0[3] = ((pixel.0[3] as f32) * a) as u8;
        });
        
        Self::from_image(img)
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &mut self.id);
        }
    }
}



pub struct TextureRenderer<'a> {
    shader: ShaderProgram,
    mesh: Mesh,
    next_vertex: Vec<f32>,
    last_tex: Option<&'a Texture>,
    combined: Option<&'a Matrix4<f32>>,
    dirty: bool,
}
impl<'a> TextureRenderer<'a> {
    pub fn new(vertex_shader_code: &str, fragment_shader_code: &str) -> Self {
        let mut shader = ShaderProgram::new();
        shader.create_vertex_shader(vertex_shader_code);
        shader.create_fragment_shader(fragment_shader_code);
        shader.link();
        
        let mesh = Mesh::new(VertexAttributes::with(true, true, false, true));
        let next = vec![0f32; mesh.attribs.vertex_size.into()];
        Self {
            shader: shader,
            mesh: mesh,
            next_vertex: next,
            last_tex: None,
            combined: None,
            dirty: false,
        }
    }
    
    pub fn begin(&mut self, combined: &'a Matrix4<f32>) {
        self.combined = Some(combined);
        
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }
    
    pub fn flush(&mut self) {
        if self.last_tex.is_none() || self.combined.is_none() { return }
        
        self.last_tex.unwrap().bind();
        
        self.shader.set_uniform1i32("textureSampler", 0);
        self.mesh.render(&self.shader, false, gl::TRIANGLES, self.combined.unwrap().clone());
        self.mesh.clear();
        self.dirty = false;
    }
    
    pub fn end(&mut self) {
        if self.dirty {
            self.flush();
        }
        
        unsafe {
            gl::Disable(gl::BLEND);
        }
        
        self.combined = None;
        self.last_tex = None;
    }
    
    //todo: Texture and TextureRegion methods
    pub fn texture_xy(&mut self, tex: &'a Texture, x: f32, y: f32) {
        self.texture(tex, x, y, tex.width as f32, tex.height as f32, 0.0, 0.0, 1.0, 1.0);
    }
    
    pub fn texture(&mut self, tex: &'a Texture, x: f32, y: f32, width: f32, height: f32, u: f32, v: f32, u2: f32, v2: f32) {
        if self.last_tex.is_none() {
            self.last_tex = Some(tex);
        }
        
        if self.last_tex.is_some() && self.last_tex.unwrap().id != tex.id {
            self.flush();
            self.last_tex = Some(tex);
        }
        
        self.dirty = true;
        
        self.color(1.0, 1.0, 1.0, 1.0);
        self.tex_coord(u, v);
        self.vertex(x, y, 0.0);
        
        self.color(1.0, 1.0, 1.0, 1.0);
        self.tex_coord(u, v2);
        self.vertex(x, y + height, 0.0);
        
        self.color(1.0, 1.0, 1.0, 1.0);
        self.tex_coord(u2, v2);
        self.vertex(x + width, y + height, 0.0);
        
        
        self.color(1.0, 1.0, 1.0, 1.0);
        self.tex_coord(u2, v2);
        self.vertex(x + width, y + height, 0.0);
        
        self.color(1.0, 1.0, 1.0, 1.0);
        self.tex_coord(u2, v);
        self.vertex(x + width, y, 0.0);
        
        self.color(1.0, 1.0, 1.0, 1.0);
        self.tex_coord(u, v);
        self.vertex(x, y, 0.0);
    }
    
    pub fn tex_coord(&mut self, u: f32, v: f32) {
        if self.mesh.attribs.has_tex_coords {
            let offset = self.mesh.get_vertex_offset(Usage::TEXCOORDS) as usize;
            self.next_vertex[offset] = u;
            self.next_vertex[offset+1] = v;
        }
    }
    
    pub fn color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        if self.mesh.attribs.has_colors {
            let offset = self.mesh.get_vertex_offset(Usage::COLORS) as usize;
            self.next_vertex[offset] = r;
            self.next_vertex[offset+1] = g;
            self.next_vertex[offset+2] = b;
            self.next_vertex[offset+3] = a;
        }
    }
    
    pub fn vertex(&mut self, x: f32, y: f32, z: f32) {
        self.next_vertex[0] = x;
        self.next_vertex[1] = y;
        self.next_vertex[2] = z;
        self.mesh.vertex(&self.next_vertex);
        
        self.next_vertex.fill(0.0);
    }
}