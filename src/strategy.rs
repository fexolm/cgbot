use crate::simulation;

use super::{
    simulation::*,
    vec2::Vec2,
    world::{BlipDirection, Drone, World},
};

pub struct Strategy {
    solver: Solver,
    cells: [[f32; 20]; 20],
}

impl Strategy {
    pub fn new() -> Self {
        Strategy {
            solver: Solver::new(),
            cells: [[1.; 20]; 20],
        }
    }
}

impl Strategy {
    pub fn play(&mut self, world: &World) {
        let mut solver = Solver::new();

        let actions = solver.find_best_action(world, &self.cells);

        for (action, drone) in actions.iter().zip(world.me.drones.values()) {
            let light = action.light as i32;
            if let Some(mov) = action.mov {
                let Vec2 { x, y } = (drone.pos + mov).clamp(
                    Vec2 { x: 0., y: 0. },
                    Vec2 {
                        x: 10000.,
                        y: 10000.,
                    },
                );

                self.cells[(y / 500.) as usize][(x / 500.) as usize] = 0.;

                if drone.bat >= 5 && action.light {
                    for dx in -1..2 {
                        for dy in -1..2 {
                            let x = (((x / 500.) as i32) + dx).clamp(0, 10);
                            let y = (((y / 500.) as i32) + dy).clamp(0, 10);
                            self.cells[y as usize][x as usize] = 0.;
                        }
                    }
                }

                println!("MOVE {x} {y} {light}");
            } else {
                let x = drone.pos.x;
                let y = (drone.pos.y - 300.).clamp(0., 10000.);

                self.cells[(y / 500.) as usize][(x / 500.) as usize] = 0.;

                if drone.bat >= 5 && action.light {
                    for dx in -1..2 {
                        for dy in -1..2 {
                            let x = (((x / 500.) as i32) + dx).clamp(0, 10);
                            let y = (((y / 500.) as i32) + dy).clamp(0, 10);
                            self.cells[y as usize][x as usize] = 0.;
                        }
                    }
                }

                println!("WAIT {light}");
            }
        }
    }
}
