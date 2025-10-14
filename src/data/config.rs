use super::color::Color;

pub struct Config {
    pub online_color: Color,
    pub offline_color: Color,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            online_color: Color::GREEN,
            offline_color: Color::RED,
        }
    }
}
