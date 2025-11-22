#[derive(Copy, Clone)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

pub struct Layout {
    pub candles_area: Area,
    pub orders_area: Area,
    pub order_book_area: Area,
    pub order_flow_area: Area,
    pub status_area: Area,
}

impl Layout {
    pub fn new(window_width: i32, window_height: i32) -> Self {
        let dom_width = 100;
        let order_flow_width = 100;
        let status_height = 20;
        let orders_width = 50;

        let status_area = Area {
            left: 0,
            top: window_height - status_height,
            width: window_width,
            height: status_height,
        };

        let candles_area = Area {
            left: 0,
            top: 0,
            width: window_width - dom_width - order_flow_width - orders_width,
            height: window_height - status_height,
        };

        let orders_area = Area {
            left: window_width - dom_width - order_flow_width - orders_width,
            top: 0,
            width: orders_width,
            height: window_height - status_height,
        };

        let order_book_area = Area {
            left: window_width - order_flow_width - dom_width,
            top: 0,
            width: dom_width,
            height: window_height - status_height,
        };

        let order_flow_area = Area {
            left: window_width - order_flow_width,
            top: 0,
            width: order_flow_width,
            height: window_height - status_height,
        };

        Self {
            candles_area,
            orders_area,
            order_book_area,
            order_flow_area,
            status_area,
        }
    }
}
