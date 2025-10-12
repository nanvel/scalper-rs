#[derive(Clone, Copy)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    pub const WHITE: Color = Color {
        a: 255,
        r: 255,
        g: 255,
        b: 255,
    };
    pub const BLACK: Color = Color {
        a: 255,
        r: 0,
        g: 0,
        b: 0,
    };
    pub const GREEN: Color = Color {
        a: 255,
        r: 0,
        g: 170,
        b: 0,
    };
    pub const RED: Color = Color {
        a: 255,
        r: 204,
        g: 0,
        b: 0,
    };
    pub const GRAY: Color = Color {
        a: 255,
        r: 51,
        g: 51,
        b: 51,
    };
}
