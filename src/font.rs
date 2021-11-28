use std::collections::HashMap;
use std::mem::swap;
use crate::graphics::{Texture, TextureRenderer};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Glyph {
    pub id: char,
    pub u: f32,
    pub v: f32,
    pub u2: f32,
    pub v2: f32,
    pub width: f32,
    pub height: f32,
    pub x_offset: i32,
    pub y_offset: i32,
    pub x_advance: i32,
}

pub struct BitmapFont {
    pub tex: Texture,
    pub glyphs: HashMap<char, Glyph>,
    pub face: String,
    pub size: u32,
    pub bold: bool,
    pub italic: bool,
    pub spacing: f32,
}
impl BitmapFont {
    pub fn new(tex: Texture, fnt_data: &str) -> Self {
        let mut font = Self {
            tex,
            glyphs: Default::default(),
            face: "".to_string(),
            size: 0,
            bold: false,
            italic: false,
            spacing: 0.0
        };
        
        for line in fnt_data.lines() {
            let line_parts: Vec<&str> = line.split_whitespace().collect();
            
            match *line_parts.first().unwrap_or(&"") {
                "info" => {
                    for part in line_parts { match part {
                        "face" => font.face = part.to_owned(),
                        "size" => font.size = part.parse().unwrap_or(0u32),
                        "bold" => font.bold = part.eq("1"),
                        "italic" => font.italic = part.eq("1"),
                        _ => ()
                    }}
                },
                "char" => {
                    let mut glyph = Glyph::default();
                    let mut x = 0f32;
                    let mut y = 0f32;
                    for part in line_parts {
                        let pair = part.split_once("=").unwrap_or((part, ""));
                        
                        match pair {
                            ("id", val) => glyph.id = char::from_u32(val.parse::<u32>().unwrap()).unwrap_or(char::from(val.parse::<u8>().unwrap())),
                            ("x", val) => x = val.parse().unwrap(),
                            ("y", val) => y = val.parse().unwrap(),
                            ("width", val) => glyph.width = val.parse().unwrap(),
                            ("height", val) => glyph.height = val.parse().unwrap(),
                            ("xoffset", val) => glyph.x_offset = val.parse().unwrap(),
                            ("yoffset", val) => glyph.y_offset = val.parse().unwrap(),
                            ("xadvance", val) => glyph.x_advance = val.parse().unwrap(),
                            _ => ()
                        }
                    }
                    
                    glyph.u = x / (font.tex.width as f32);
                    glyph.v = y / (font.tex.height as f32);
                    glyph.u2 = (x + glyph.width) / (font.tex.width as f32);
                    glyph.v2 = (y + glyph.height) / (font.tex.height as f32);
                    
                    glyph.v = -glyph.v + (font.tex.height as f32);
                    glyph.v2 = -glyph.v2 + (font.tex.height as f32);
                    
                    swap(&mut glyph.v, &mut glyph.v2);
                    
                    font.glyphs.insert(glyph.id, glyph);
                },
                _ => ()
            }
        }
        
        font
    }
    
    pub fn render<'a>(&'a self, tr: &mut TextureRenderer<'a>, text: &str, x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) {
        let mut curx = x;
        for c in text.chars() {
            if let Some(glyph) = self.glyphs.get(&c) {
                tr.texture(&self.tex, curx + (glyph.x_offset as f32), y, glyph.width, glyph.height, glyph.u, glyph.v, glyph.u2, glyph.v2, r, g, b, a);
                curx += (glyph.x_advance as f32) + self.spacing;
            }
        }
        tr.flush();
    }
}




