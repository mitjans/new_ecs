use super::{grid_position::GridPosition, wagon::Wagon};
use crate::{resources::tile_size::TileSize, Transformer};
use new_ecs::World;
use raylib::prelude::*;
use std::collections::VecDeque;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy, Default)]
pub enum Direction {
    #[default]
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn get_direction_from_positions(from: GridPosition, to: GridPosition) -> Self {
        let drow = to.row as i32 - from.row as i32;
        let dcol = to.col as i32 - from.col as i32;

        if drow == 1 {
            return Self::South;
        }

        if drow == -1 {
            return Self::North;
        }

        if dcol == 1 {
            return Self::East;
        }

        if dcol == -1 {
            return Self::West;
        }

        panic!();
    }
}

#[derive(Default)]
pub struct Train {
    pub coordinates: Vector2,
    pub stations: Vec<GridPosition>,
    pub next_station: usize,
    pub route: VecDeque<GridPosition>,
    pub direction: Direction,
    pub last_position: Option<GridPosition>,
    pub wagons: Vec<Wagon>,
    pub elapsed: f32,
}

impl Train {
    pub fn draw(&self, world: &World, context: &mut RaylibDrawHandle) {
        let tile_size = world.get_resource::<TileSize>().unwrap();

        let position = Transformer::position(world, self.coordinates).unwrap();
        let coordinates =
            Transformer::coordinate(world, position, crate::components::anchor::Anchor::TopLeft);

        if self.direction == Direction::North {
            context.draw_triangle(
                Vector2 {
                    x: coordinates.x,
                    y: coordinates.y + tile_size.0 as f32,
                },
                Vector2 {
                    x: coordinates.x + tile_size.0 as f32,
                    y: coordinates.y + tile_size.0 as f32,
                },
                Vector2 {
                    x: coordinates.x + tile_size.0 as f32 / 2f32,
                    y: coordinates.y,
                },
                Color::RED,
            );
        } else if self.direction == Direction::South {
            context.draw_triangle(
                Vector2 {
                    x: coordinates.x,
                    y: coordinates.y,
                },
                Vector2 {
                    x: coordinates.x + tile_size.0 as f32 / 2f32,
                    y: coordinates.y + tile_size.0 as f32,
                },
                Vector2 {
                    x: coordinates.x + tile_size.0 as f32,
                    y: coordinates.y,
                },
                Color::RED,
            );
        } else if self.direction == Direction::East {
            context.draw_triangle(
                Vector2 {
                    x: coordinates.x,
                    y: coordinates.y,
                },
                Vector2 {
                    x: coordinates.x,
                    y: coordinates.y + tile_size.0 as f32,
                },
                Vector2 {
                    x: coordinates.x + tile_size.0 as f32,
                    y: coordinates.y + tile_size.0 as f32 / 2f32,
                },
                Color::RED,
            );
        } else if self.direction == Direction::West {
            context.draw_triangle(
                Vector2 {
                    x: coordinates.x + tile_size.0 as f32,
                    y: coordinates.y,
                },
                Vector2 {
                    x: coordinates.x,
                    y: coordinates.y + tile_size.0 as f32 / 2f32,
                },
                Vector2 {
                    x: coordinates.x + tile_size.0 as f32,
                    y: coordinates.y + tile_size.0 as f32,
                },
                Color::RED,
            );
        }
    }
}
