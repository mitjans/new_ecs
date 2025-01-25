use std::collections::{BinaryHeap, HashSet, VecDeque};

use crate::{
    components::{tile::Tile, train::Direction},
    schedulers::update_scheduler::UpdateSystem,
    Train, Transformer,
};

use raylib::prelude::*;

use crate::{GridPosition, World};

#[derive(Default)]
pub struct TrainRouteSystem;

impl UpdateSystem for TrainRouteSystem {
    fn update(&mut self, world: &mut World, _: &mut RaylibHandle) {
        let train_query = world.query().with_component::<Train>().iter(world);
        let tile_results = world
            .query()
            .with_component::<Tile>()
            .iter(world)
            .collect::<Vec<_>>();
        let tiles = tile_results
            .iter()
            .map(|result| result.get::<Tile>().unwrap())
            .collect::<Vec<_>>();

        for mut result in train_query {
            let train = result.get_mut::<Train>().unwrap();

            if train.elapsed != 0f32 {
                continue;
            }

            let current_grid_position = Transformer::position(world, train.coordinates).unwrap();

            let mut next_station = train.stations[train.next_station];

            if current_grid_position == next_station {
                // We arrived at station! Calculate next station
                train.next_station = (train.next_station + 1) % train.stations.len();
                next_station = train.stations[train.next_station];
            }

            train.route = self.path_between(
                tiles.as_slice(),
                current_grid_position,
                next_station,
                train.direction,
            );
        }
    }
}

impl TrainRouteSystem {
    fn neighbor_tiles(&self, tiles: &[&Tile], position: GridPosition) -> Vec<GridPosition> {
        tiles
            .iter()
            .filter_map(|tile| {
                if tile.get_connections().0.contains(&position) {
                    Some(tile.get_position())
                } else {
                    None
                }
            })
            .collect()
    }

    fn path_between(
        &self,
        rails: &[&Tile],
        from: GridPosition,
        to: GridPosition,
        direction: Direction,
    ) -> VecDeque<GridPosition> {
        #[derive(PartialEq, Eq, Debug)]
        struct Node {
            position: GridPosition,
            weight: u32,
            route: VecDeque<GridPosition>,
            direction: Direction,
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.weight.cmp(&other.weight).reverse()
            }
        }

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut visited = HashSet::new();

        let mut heap = BinaryHeap::new();
        heap.push(Node {
            position: from,
            route: VecDeque::new(),
            weight: 0,
            direction,
        });
        while let Some(node) = heap.pop() {
            if visited.contains(&(node.position, node.direction)) {
                continue;
            }

            visited.insert((node.position, node.direction));

            if node.position == to {
                return node.route;
            }

            for position in self.neighbor_tiles(rails, node.position) {
                let route = [Vec::from(node.route.clone()), vec![position]].concat();
                if Direction::get_direction_from_positions(position, node.position)
                    != node.direction
                {
                    heap.push(Node {
                        position,
                        route: VecDeque::from(route),
                        weight: node.weight + node.position.manhattan(&to) as u32,
                        direction: Direction::get_direction_from_positions(node.position, position),
                    });
                }
            }
        }

        VecDeque::default()
    }
}
