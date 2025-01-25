mod components;
mod render_systems;
mod resources;
mod schedulers;
mod update_systems;

use components::{
    anchor::Anchor, connections::Connections, ghost::Ghost, grid_position::GridPosition,
    rail::Rail, train::Train, wagon::Wagon,
};
use new_ecs::World;
use raylib::prelude::*;
use render_systems::{
    render_ghost_system::RenderGhostSystem, render_grid_system::RenderGridSystem,
    render_tile_system::RenderTileSystem, render_train_system::RenderTrainSystem,
};
use resources::{
    grid_divisions::GridDivisions, tile_size::TileSize, world_height::WorldHeight,
    world_width::WorldWidth,
};
use schedulers::{draw_scheduler::DrawScheduler, update_scheduler::UpdateScheduler};
use update_systems::{
    ghost_cursor_system::GhostCursorSystem, input_system::InputHandlerSystem,
    train_move_system::TrainMoveSystem, train_route_system::TrainRouteSystem,
};

struct Transformer;

impl Transformer {
    pub fn position(world: &World, coordinate: Vector2) -> Option<GridPosition> {
        let tile_size = world.get_resource::<TileSize>().unwrap();
        let world_width = world.get_resource::<WorldWidth>().unwrap();
        let world_height = world.get_resource::<WorldHeight>().unwrap();

        if coordinate.x < 0f32
            || coordinate.y < 0f32
            || coordinate.x >= world_width.0 as f32
            || coordinate.y >= world_height.0 as f32
        {
            return None;
        }

        let (row, col) = (
            (coordinate.y / tile_size.0 as f32) as usize,
            (coordinate.x / tile_size.0 as f32) as usize,
        );

        Some(GridPosition { row, col })
    }

    pub fn coordinate(world: &World, position: GridPosition, anchor: Anchor) -> Vector2 {
        let tile_size = world.get_resource::<TileSize>().unwrap();

        let (mut x, mut y) = (
            position.col as f32 * tile_size.0 as f32,
            position.row as f32 * tile_size.0 as f32,
        );

        match anchor {
            Anchor::Center => {
                x += tile_size.0 as f32 / 2f32;
                y += tile_size.0 as f32 / 2f32;
            }
            Anchor::TopLeft => (),
        }

        Vector2 { x, y }
    }
}

fn main() {
    let divisions = GridDivisions(10);
    let world_width = WorldWidth(400);
    let world_height = WorldHeight(400);
    let tile_size = TileSize(world_width.0 / divisions.0 as i32);

    let mut world = World::default();

    {
        world.add_resource(world_width);
        world.add_resource(world_height);
        world.add_resource(divisions);
        world.add_resource(tile_size);
        world.add_resource(tile_size);
    }

    {
        world
            .create_entity()
            .with_component(Ghost::default())
            .spawn();

        let coordinates = Transformer::coordinate(&world, GridPosition::new(1, 1), Anchor::Center);
        world
            .create_entity()
            .with_component(Train {
                coordinates,
                stations: vec![
                    GridPosition::new(4, 4),
                    GridPosition::new(0, 0),
                    GridPosition::new(5, 5),
                ],
                wagons: vec![
                    Wagon {
                        position: GridPosition::new(2, 1),
                    },
                    Wagon {
                        position: GridPosition::new(3, 1),
                    },
                ],
                ..Default::default()
            })
            .spawn();

        let coordinates = Transformer::coordinate(&world, GridPosition::new(2, 2), Anchor::Center);
        world
            .create_entity()
            .with_component(Train {
                coordinates,
                stations: vec![
                    GridPosition::new(5, 5),
                    GridPosition::new(1, 1),
                    GridPosition::new(6, 6),
                ],
                wagons: vec![
                    Wagon {
                        position: GridPosition::new(3, 2),
                    },
                    Wagon {
                        position: GridPosition::new(4, 2),
                    },
                ],
                ..Default::default()
            })
            .spawn();

        let coordinates = Transformer::coordinate(&world, GridPosition::new(3, 3), Anchor::Center);
        world
            .create_entity()
            .with_component(Train {
                coordinates,
                stations: vec![
                    GridPosition::new(6, 6),
                    GridPosition::new(2, 2),
                    GridPosition::new(7, 7),
                ],
                wagons: vec![
                    Wagon {
                        position: GridPosition::new(4, 3),
                    },
                    Wagon {
                        position: GridPosition::new(5, 3),
                    },
                ],
                ..Default::default()
            })
            .spawn();
    }

    let mut draw_scheduler = DrawScheduler::default();

    {
        draw_scheduler.add_system(RenderGridSystem);
        draw_scheduler.add_system(RenderTileSystem);
        draw_scheduler.add_system(RenderGhostSystem);
        draw_scheduler.add_system(RenderTrainSystem);
    }

    let mut update_scheduler = UpdateScheduler::default();
    {
        update_scheduler.add_system(InputHandlerSystem::default());
        update_scheduler.add_system(GhostCursorSystem);
        update_scheduler.add_system(TrainRouteSystem);
        update_scheduler.add_system(TrainMoveSystem);
    }

    let (mut rl, thread) = raylib::init()
        .size(world_width.0, world_height.0)
        .title("trains")
        .build();

    {
        // rl.set_target_fps(60);
    }

    while !rl.window_should_close() {
        update_scheduler.update(&mut world, &mut rl);

        let entities = world.entity_index.len();

        let mut canvas = rl.begin_drawing(&thread);

        draw_scheduler.render(&mut world, &mut canvas);

        canvas.draw_text(&format!("{}", entities), 10, 10, 20, Color::GOLDENROD);

        canvas.clear_background(Color::WHITE);

        canvas.draw_text(
            &format!("{}", canvas.get_fps()),
            300,
            10,
            20,
            Color::GOLDENROD,
        );
    }
}
