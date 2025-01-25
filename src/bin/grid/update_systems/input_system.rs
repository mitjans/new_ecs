use crate::{
    components::{ghost::Ghost, station::Station, tile::Tile},
    schedulers::update_scheduler::UpdateSystem,
    Connections, Rail,
};

use raylib::prelude::*;

use crate::{GridPosition, Transformer, World};

#[derive(Default)]
pub struct InputHandlerSystem {
    last_position: Option<GridPosition>,
}

impl UpdateSystem for InputHandlerSystem {
    fn update(&mut self, world: &mut World, context: &mut RaylibHandle) {
        let mouse_position = context.get_mouse_position();
        let clicked_position = Transformer::position(world, mouse_position);

        let mut ghost_query = world.query().with_component::<Ghost>().iter(world);

        let mut ghost = ghost_query.next().unwrap();
        let ghost = ghost.get_mut::<Ghost>().unwrap();

        match context.get_key_pressed() {
            Some(KeyboardKey::KEY_C) => {
                // self.clear_grid(world);
                // self.reset_trains(world);
            }
            Some(KeyboardKey::KEY_S) => {
                ghost.selected_tile = Tile::Station(Station {
                    color: Color::RED.alpha(0.5),
                    ..Default::default()
                })
            }
            Some(KeyboardKey::KEY_R) => {
                ghost.selected_tile = Tile::Rail(Rail {
                    color: Color::BLACK.alpha(0.5),
                    ..Default::default()
                })
            }
            _ => (),
        }

        let mut spawned_station = false;

        if context.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(clicked_position) = clicked_position {
                self.last_position = Some(clicked_position);

                if let Tile::Station(_) = &ghost.selected_tile {
                    let station = Station::default();
                    self.spawn_tile(world, clicked_position, Tile::Station(station));
                    spawned_station = true;
                }
            }
        } else if context.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(clicked_position) = clicked_position {
                if let Tile::Station(_) = &ghost.selected_tile {
                    let station = Station::default();
                    self.spawn_tile(world, clicked_position, Tile::Station(station));
                    spawned_station = true;
                }
                self.last_position = None;
            }
        }

        if context.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(clicked_position) = clicked_position {
                if !spawned_station {
                    self.spawn_tile(world, clicked_position, Tile::Rail(Rail::default()));
                }
                self.last_position = Some(clicked_position);
            }
        }

        if context.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            if let Some(_clicked_position) = clicked_position {
                // self.despawn_rail(world, clicked_position);
            }
        }
    }
}

impl InputHandlerSystem {
    fn spawn_tile(&self, world: &mut World, clicked_position: GridPosition, tile: Tile) {
        let query = world.query().with_component::<Tile>().iter(world);

        let mut new_connections = Connections::default();
        for mut result in query {
            let tile = result.get_mut::<Tile>().unwrap();
            let tile_position = tile.get_position();

            if self
                .last_position
                .is_some_and(|last_position| last_position == tile_position)
                && self
                    .last_position
                    .is_some_and(|last_position| last_position.manhattan(&clicked_position) != 0)
            {
                tile.get_connections_mut().0.insert(clicked_position);
                new_connections.0.insert(tile_position);
            }
        }

        let mut query = world
            .query()
            .with_component::<Tile>()
            .iter(world)
            .collect::<Vec<_>>();

        let overlap_index = query.iter().enumerate().find_map(|(index, result)| {
            let tile = result.get::<Tile>().unwrap();
            let tile_position = tile.get_position();

            if tile_position == clicked_position {
                return Some(index);
            }
            None
        });

        if let Some(index) = overlap_index {
            let overlap_tile = query.get_mut(index).unwrap().get_mut::<Tile>().unwrap();
            match tile {
                Tile::Rail(_) => {
                    let tile_connections = overlap_tile.get_connections_mut();
                    tile_connections.0.extend(new_connections.0);
                }
                Tile::Station(_) => {
                    *overlap_tile = Tile::Station(Station {
                        connections_offset: 0.25,
                        color: Color::RED,
                        connections: overlap_tile.get_connections_mut().clone(),
                        position: clicked_position,
                        connections_color: Color::BLACK,
                    });
                }
            }
        } else {
            match tile {
                Tile::Rail(_) => world
                    .create_entity()
                    .with_component(Tile::Rail(Rail {
                        color: Color::BLACK,
                        connections: new_connections,
                        position: clicked_position,
                    }))
                    .spawn(),
                Tile::Station(_) => world
                    .create_entity()
                    .with_component(Tile::Station(Station {
                        connections_offset: 0.25,
                        color: Color::RED,
                        connections: new_connections,
                        position: clicked_position,
                        connections_color: Color::BLACK,
                    }))
                    .spawn(),
            };
        }
    }

    // fn despawn_rail(&self, world: &mut World, clicked_position: GridPosition) {
    //     let (result, entity_ids) = world.query().with_component::<Tile>().run();

    //     let overlapped = entity_ids.iter().enumerate().any(|(index, _)| {
    //         let tile = result.first().unwrap().get(index).unwrap().borrow();
    //         let tile = tile.downcast_ref::<Tile>().unwrap();
    //         let tile_position = tile.get_position();
    //         tile_position == clicked_position
    //     });

    //     if !overlapped {
    //         return;
    //     }

    //     let mut entity_id: Option<usize> = None;
    //     entity_ids.iter().enumerate().for_each(|(index, id)| {
    //         let mut tile = result.first().unwrap().get(index).unwrap().borrow_mut();
    //         let tile = tile.downcast_mut::<Tile>().unwrap();
    //         let tile_position = tile.get_position();
    //         let tile_connectiosn = tile.get_connections_mut();
    //         if tile_position == clicked_position {
    //             entity_id = Some(*id);
    //         }

    //         tile_connectiosn.0.remove(&clicked_position);
    //     });

    //     if let Some(id) = entity_id {
    //         world.remove_entity(id);
    //     }
    // }

    // fn clear_grid(&self, world: &mut World) {
    //     let (_, entity_ids) = world.query().with_component::<Tile>().run();
    //     entity_ids
    //         .iter()
    //         .for_each(|entity_id| world.remove_entity(*entity_id));
    // }

    // fn reset_trains(&self, world: &mut World) {
    //     let (_, entity_ids) = world.query().with_component::<Train>().run();
    //     entity_ids
    //         .iter()
    //         .for_each(|entity_id| world.remove_entity(*entity_id));
    // }
}
