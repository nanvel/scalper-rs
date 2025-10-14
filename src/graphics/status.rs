use crate::data::Config;
use raqote::DrawTarget;

pub struct StatusRenderer {
    width: i32,
    height: i32,
    offset_left: i32,
    offset_top: i32,
    padding: i32,
}

impl StatusRenderer {
    pub fn new(width: i32, height: i32, offset_left: i32, offset_top: i32) -> Self {
        Self {
            width,
            height,
            offset_left,
            offset_top,
            padding: 1,
        }
    }

    pub fn render(&mut self, symbol: &str, dt: &mut DrawTarget, config: &Config) {}
}
