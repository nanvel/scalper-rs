use super::color::Color;

pub struct Config {
    pub online_color: Color,
    pub offline_color: Color,
    pub bullish_color: Color,
    pub bearish_color: Color,
    pub background_color: Color,
    pub text_color: Color,

    pub dom_width: i32,
    pub status_height: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            online_color: Color::GREEN,
            offline_color: Color::RED,
            bullish_color: Color::GREEN,
            bearish_color: Color::RED,
            background_color: Color::WHITE,
            text_color: Color::BLACK,
            dom_width: 100,
            status_height: 20,
        }
    }
}
