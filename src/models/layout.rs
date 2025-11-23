#[derive(Copy, Clone, Debug)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

pub struct Layout {
    pub width: i32,
    pub height: i32,
    pub candles_area: Area,
    pub orders_area: Area,
    pub order_book_area: Area,
    pub order_flow_area: Area,
    pub status_area: Area,
}

impl Layout {
    pub fn new(width: i32, height: i32) -> Self {
        let dom_width = 100;
        let order_flow_width = 100;
        let status_height = 20;
        let orders_width = 50;

        let status_area = Area {
            left: 0,
            top: height - status_height,
            width,
            height: status_height,
        };

        let candles_area = Area {
            left: 0,
            top: 0,
            width: width - dom_width - order_flow_width - orders_width,
            height: height - status_height,
        };

        let orders_area = Area {
            left: width - dom_width - order_flow_width - orders_width,
            top: 0,
            width: orders_width,
            height: height - status_height,
        };

        let order_book_area = Area {
            left: width - order_flow_width - dom_width,
            top: 0,
            width: dom_width,
            height: height - status_height,
        };

        let order_flow_area = Area {
            left: width - order_flow_width,
            top: 0,
            width: order_flow_width,
            height: height - status_height,
        };

        Self {
            width,
            height,
            candles_area,
            orders_area,
            order_book_area,
            order_flow_area,
            status_area,
        }
    }
}
