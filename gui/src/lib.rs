pub mod circle;
pub mod common;
pub mod components;
pub mod display;
pub mod draw;
pub mod font;
pub mod page;
pub mod text;

pub use epd_waveshare::color::Color::{Black as White, White as Black};
pub use epd_waveshare::epd7in5_v2::{HEIGHT, WIDTH};

pub const GRAY_LUMA: u8 = 127;
