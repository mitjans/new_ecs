use raylib::color::Color;

use super::{rail::Rail, tile::Tile};

#[derive(Debug)]
pub struct Ghost {
    pub selected_tile: Tile,
}

impl Default for Ghost {
    fn default() -> Self {
        Self {
            selected_tile: Tile::Rail(Rail {
                color: Color::BLACK.alpha(0.5),
                ..Default::default()
            }),
        }
    }
}
