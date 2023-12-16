#![allow(unused_variables)]
#![allow(dead_code)]

#[cfg(feature = "rewind")]
use crate::rewind_client::RewindClient;

pub struct Debug {
    #[cfg(feature = "rewind")]
    client: RewindClient,
}

pub const RED: u32 = 0xff0000;
pub const GREEN: u32 = 0x00ff00;
pub const BLUE: u32 = 0x0000ff;
pub const DARK_RED: u32 = 0x770000;
pub const DARK_GREEN: u32 = 0x007700;
pub const DARK_BLUE: u32 = 0x000077;
pub const TRANSPARENT: u32 = 0x7f000000;
pub const INVISIBLE: u32 = 0x01000000;

impl Debug {
    pub fn new() -> Debug {
        Debug {
            #[cfg(feature = "rewind")]
            client: RewindClient::new().unwrap(),
        }
    }

    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: u32) {
        #[cfg(feature = "rewind")]
        self.client.line(x1, y1, x2, y2, color).unwrap();
    }

    pub fn polyline(&mut self, points: &[f64], color: u32) {
        #[cfg(feature = "rewind")]
        self.client.polyline(points, color).unwrap();
    }

    pub fn circle(&mut self, x: f64, y: f64, radius: f64, color: u32, fill: bool) {
        #[cfg(feature = "rewind")]
        self.client.circle(x, y, radius, color, fill).unwrap();
    }

    pub fn rectangle(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: u32, fill: bool) {
        #[cfg(feature = "rewind")]
        self.client.rectangle(x1, y1, x2, y2, color, fill).unwrap();
    }

    pub fn rectangle_with_colors(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        colors: [u32; 4],
        fill: bool,
    ) {
        #[cfg(feature = "rewind")]
        self.client
            .rectangle_with_colors(x1, y1, x2, y2, colors, fill)
            .unwrap();
    }

    pub fn triangle(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        color: u32,
        fill: bool,
    ) {
        #[cfg(feature = "rewind")]
        self.client
            .triangle(x1, y1, x2, y2, x3, y3, color, fill)
            .unwrap();
    }

    pub fn triangle_with_colors(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        colors: [u32; 3],
        fill: bool,
    ) {
        #[cfg(feature = "rewind")]
        self.client
            .triangle_with_colors(x1, y1, x2, y2, x3, y3, colors, fill)
            .unwrap();
    }

    pub fn circle_popup(&mut self, x: f64, y: f64, radius: f64, text: &str) {
        #[cfg(feature = "rewind")]
        self.client.circle_popup(x, y, radius, text).unwrap();
    }

    pub fn rectangle_popup(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, text: &str) {
        #[cfg(feature = "rewind")]
        self.client.rectangle_popup(x1, y1, x2, y2, text).unwrap();
    }

    pub fn options(&mut self, layer: i32, permanent: bool) {
        #[cfg(feature = "rewind")]
        self.client.options(layer, permanent).unwrap();
    }

    pub fn message(&mut self, text: &str) {
        #[cfg(feature = "rewind")]
        self.client.message(text).unwrap();
    }

    pub fn end_frame(&mut self) {
        #[cfg(feature = "rewind")]
        self.client.end_frame().unwrap();
    }
}
