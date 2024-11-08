pub mod common;
pub mod components;
pub mod display;
pub mod draw;
pub mod font;
pub mod page;
pub mod text;

pub use epd_waveshare::color::Color::{Black as White, White as Black};

pub const GRAY_LUMA: u8 = 127;
