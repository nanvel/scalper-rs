use raqote::SolidSource;

#[derive(Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for SolidSource {
    fn from(color: Color) -> Self {
        SolidSource::from_unpremultiplied_argb(color.a, color.r, color.g, color.b)
    }
}
