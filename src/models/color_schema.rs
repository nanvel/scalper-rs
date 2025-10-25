use crate::models::Color;
use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

pub struct ColorSchema {
    pub theme: Theme,

    pub background: Color,
    pub status_bar_background: Color,

    pub bullish_candle: Color,
    pub bearish_candle: Color,
    pub bid_bar: Color,
    pub ask_bar: Color,
    pub volume_buy: Color,
    pub volume_sell: Color,
    pub open_interest: Color,

    pub text_light: Color,
    pub text_dark: Color,
    pub text_negative_pnl: Color,
    pub text_positive_pnl: Color,

    pub border: Color,
    pub crosshair: Color,
    pub scale_bar: Color,
}

impl ColorSchema {
    pub fn dark() -> Self {
        Self {
            theme: Theme::Dark,

            // Base colors
            background: Color::new(17, 24, 39, 255), // #111827
            status_bar_background: Color::new(31, 41, 55, 255), // #1F2937

            // Candles
            bullish_candle: Color::new(34, 197, 94, 255), // #22C55E
            bearish_candle: Color::new(239, 68, 68, 255), // #EF4444

            // Order book bars
            bid_bar: Color::new(34, 197, 94, 200), // #22C55E
            ask_bar: Color::new(239, 68, 68, 200), // #EF4444

            volume_buy: Color::new(34, 197, 94, 128), // #22C55E at 50% opacity
            volume_sell: Color::new(239, 68, 68, 128), // #EF4444 at 50% opacity

            open_interest: Color::new(148, 163, 184, 128), // #94A3B8 at 50% opacity

            // Text colors
            text_light: Color::new(229, 231, 235, 255), // #E5E7EB
            text_dark: Color::new(31, 41, 55, 255),     // #1F2937
            text_negative_pnl: Color::new(248, 113, 113, 255), // #F87171
            text_positive_pnl: Color::new(52, 211, 153, 255), // #34D399

            // UI elements
            border: Color::new(55, 65, 81, 255),       // #374151
            crosshair: Color::new(156, 163, 175, 255), // #9CA3AF
            scale_bar: Color::new(139, 92, 246, 255),  // #8B5CF6
        }
    }

    pub fn light() -> Self {
        Self {
            theme: Theme::Light,

            // Base colors
            background: Color::new(255, 255, 255, 255), // #FFFFFF
            status_bar_background: Color::new(248, 249, 250, 255), // #F8F9FA

            // Candles
            bullish_candle: Color::new(16, 185, 129, 255), // #10B981
            bearish_candle: Color::new(239, 68, 68, 255),  // #EF4444

            // Order book bars
            bid_bar: Color::new(16, 185, 129, 200), // #10B981
            ask_bar: Color::new(239, 68, 68, 200),  // #EF4444

            volume_buy: Color::new(16, 185, 129, 128), // #10B981 at 50% opacity
            volume_sell: Color::new(239, 68, 68, 128), // #EF4444 at 50% opacity

            open_interest: Color::new(100, 116, 139, 128), // #64748B at 50% opacity

            // Text colors
            text_light: Color::new(31, 41, 55, 255), // #1F2937
            text_dark: Color::new(249, 250, 251, 255), // #F9FAFB
            text_negative_pnl: Color::new(220, 38, 38, 255), // #DC2626
            text_positive_pnl: Color::new(5, 150, 105, 255), // #059669

            // UI elements
            border: Color::new(229, 231, 235, 255), // #E5E7EB
            crosshair: Color::new(107, 114, 128, 255), // #6B7280
            scale_bar: Color::new(139, 92, 246, 255), // #8B5CF6
        }
    }

    pub fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self::light(),
            Theme::Dark => Self::dark(),
            Theme::Auto => {
                let hour = Local::now().hour();
                if hour >= 20 || hour < 6 {
                    Self::dark()
                } else {
                    Self::light()
                }
            }
        }
    }
}
