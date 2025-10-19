use super::config::Config;

pub struct Area {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

pub struct Layout {
    pub candles_area: Area,
    pub dom_area: Area,
    pub order_flow_area: Area,
    pub status_area: Area,
}

impl Layout {
    pub fn new(window_width: i32, window_height: i32, config: &Config) -> Self {
        let status_area = Area {
            left: 0,
            top: window_height - config.status_height,
            width: window_width,
            height: config.status_height,
        };

        let candles_area = Area {
            left: 0,
            top: 0,
            width: window_width - config.dom_width - config.order_flow_width,
            height: window_height - config.status_height,
        };

        let dom_area = Area {
            left: window_width - config.order_flow_width - config.dom_width,
            top: 0,
            width: config.dom_width,
            height: window_height - config.status_height,
        };

        let order_flow_area = Area {
            left: window_width - config.order_flow_width,
            top: 0,
            width: config.order_flow_width,
            height: window_height - config.status_height,
        };

        Self {
            candles_area,
            dom_area,
            order_flow_area,
            status_area,
        }
    }
}
