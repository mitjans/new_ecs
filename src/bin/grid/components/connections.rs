use super::grid_position::GridPosition;
use std::collections::HashSet;

#[derive(Default, Debug, Clone)]
pub struct Connections(pub HashSet<GridPosition>);
