use new_ecs::World;
use raylib::prelude::RaylibDrawHandle;

use super::{connections::Connections, grid_position::GridPosition, rail::Rail, station::Station};

#[derive(Debug)]
pub enum Tile {
    Rail(Rail),
    Station(Station),
}

impl Tile {
    pub fn draw(&self, world: &World, context: &mut RaylibDrawHandle) {
        match self {
            Tile::Rail(rail) => rail.draw(world, context),
            Tile::Station(station) => station.draw(world, context),
        }
    }

    pub fn set_position(&mut self, position: GridPosition) {
        match self {
            Tile::Rail(rail) => rail.position = position,
            Tile::Station(station) => station.position = position,
        }
    }

    pub fn get_position(&self) -> GridPosition {
        match self {
            Tile::Station(station) => station.position,
            Tile::Rail(rail) => rail.position,
        }
    }

    pub fn get_connections(&self) -> &Connections {
        match self {
            Tile::Rail(rail) => &rail.connections,
            Tile::Station(station) => &station.connections,
        }
    }

    pub fn get_connections_mut(&mut self) -> &mut Connections {
        match self {
            Tile::Rail(rail) => &mut rail.connections,
            Tile::Station(station) => &mut station.connections,
        }
    }
}
