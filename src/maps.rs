use super::{bounds_detector::BoundsDetector, vec2::Vec2, world::World};

const E_CELLS: usize = 20;
const E_CELL_SIZE: usize = 10000 / E_CELLS;

const S_CELLS: usize = 20;
const S_CELL_SIZE: usize = 10000 / S_CELLS;

pub struct ExplorationMap {
    map: [[f32; E_CELLS]; E_CELLS],
}

pub struct ScoreMap {
    map: [[f32; S_CELLS]; S_CELLS],
}

fn position_to_grid_cell(pos: Vec2, cell_size: usize) -> (usize, usize) {
    (pos.x as usize / cell_size, pos.y as usize / cell_size)
}

impl ExplorationMap {
    pub fn new() -> Self {
        ExplorationMap {
            map: [[1.; E_CELLS]; E_CELLS],
        }
    }

    const STEP: f32 = 0.05;

    pub fn update(&mut self, world: &World) {
        for x in 0..E_CELLS {
            for y in 0..E_CELLS {
                if self.map[x][y] < 1. {
                    self.map[x][y] += Self::STEP;
                }
            }
        }

        for drone in world.me.drones.values() {
            let (x, y) = position_to_grid_cell(drone.pos, E_CELL_SIZE);

            self.map[x][y] = 0.;
        }
    }

    pub fn use_light(&mut self, pos: Vec2) {
        let (x, y) = position_to_grid_cell(pos, E_CELL_SIZE);
        let (x, y) = (x as i32, y as i32);

        for dx in -1..2 {
            for dy in -1..2 {
                if x + dx >= 0
                    && x + dx <= E_CELLS as i32
                    && y + dy >= 0
                    && y + dy <= E_CELLS as i32
                {
                    self.map[(x + dx) as usize][(y + dy) as usize] = 0.;
                }
            }
        }
    }

    pub fn get_score(&self, pos: Vec2) -> f32 {
        let (x, y) = position_to_grid_cell(pos, E_CELL_SIZE);
        self.map[x][y]
    }
}

impl ScoreMap {
    pub fn new() -> Self {
        ScoreMap {
            map: [[0.; S_CELLS]; S_CELLS],
        }
    }

    fn is_in_someone_scan(id: i32, world: &World) -> bool {
        world
            .me
            .drones
            .values()
            .into_iter()
            .any(|drone| drone.scans.contains(&id))
    }

    fn is_alive(id: i32, world: &World) -> bool {
        world
            .me
            .drones
            .values()
            .any(|drone| drone.blips.contains_key(&id))
    }

    pub fn update(&mut self, world: &World, bounds_detector: &BoundsDetector) {
        for x in 0..S_CELLS {
            for y in 0..S_CELLS {
                self.map[x][y] = 0.;
            }
        }

        let creatures = world.creatures.values().filter(|c| {
            c.typ != -1 && Self::is_in_someone_scan(c.id, world) && Self::is_alive(c.id, world)
        });

        for c in creatures {
            let bounds = bounds_detector.get_bounds(c.id);

            // TODO: calculate creature cost
            let creature_cost = 3.;

            let (start_x, start_y) = position_to_grid_cell(bounds.top_left, S_CELL_SIZE);
            let (end_x, end_y) = position_to_grid_cell(bounds.bot_right, S_CELL_SIZE);
            let (end_x, end_y) = ((end_x + 1).min(S_CELLS), (end_y + 1).min(S_CELLS));

            let cells_count = (end_x - start_x) * (end_y - start_y);

            for x in start_x..end_x {
                for y in start_y..end_y {
                    self.map[x][y] += creature_cost / (cells_count as f32)
                }
            }
        }
    }

    pub fn get_score(&self, pos: Vec2) -> f32 {
        let (x, y) = position_to_grid_cell(pos, S_CELL_SIZE);
        self.map[x][y]
    }
}
