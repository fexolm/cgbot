use std::collections::HashMap;

use crate::world::BlipDirection;

use super::vec2::Vec2;
use super::world::World;

#[derive(Debug)]
pub struct Bounds {
    pub top_left: Vec2,
    pub bot_right: Vec2,
}

impl Bounds {
    fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Bounds {
            top_left: Vec2::new(x1, y1),
            bot_right: Vec2::new(x2, y2),
        }
    }

    fn intersect(&mut self, other: &Bounds) {
        self.top_left = self.top_left.max(other.top_left);
        self.bot_right = self.bot_right.min(other.bot_right);
    }

    fn extend(&mut self, size: f32) {
        self.top_left.x -= size;
        self.top_left.y -= size;
        self.bot_right.x += size;
        self.bot_right.y += size;
    }

    pub fn get_center(&self) -> Vec2 {
        (self.bot_right + self.top_left) * 0.5
    }
}

pub struct BoundsDetector {
    pub bounds: HashMap<i32, Bounds>,
}

fn get_bounds_for_type(typ: i8) -> Bounds {
    match typ {
        -1 => Bounds::new(0., 0., 10000., 10000.),
        0 => Bounds::new(0., 2500., 10000., 5000.),
        1 => Bounds::new(0., 5000., 10000., 7500.),
        2 => Bounds::new(0., 7500., 10000., 10000.),
        _ => unreachable!(),
    }
}

fn get_directional_bounds(dir: BlipDirection, pos: Vec2) -> Bounds {
    match dir {
        BlipDirection::TL => Bounds::new(0., 0., pos.x, pos.y),
        BlipDirection::TR => Bounds::new(pos.x, 0., 10000., pos.y),
        BlipDirection::BL => Bounds::new(0., pos.y, pos.x, 10000.),
        BlipDirection::BR => Bounds::new(pos.x, pos.y, 10000., 10000.),
    }
}

impl BoundsDetector {
    pub fn new() -> Self {
        BoundsDetector {
            bounds: HashMap::new(),
        }
    }

    fn initialize(&mut self, world: &World) {
        if self.bounds.is_empty() {
            for c in world.creatures.values() {
                self.bounds.insert(c.id, get_bounds_for_type(c.typ));
            }
        }
    }

    fn extend_bounds(&mut self, world: &World) {
        for c in world.creatures.values() {
            let bounds = self.bounds.get_mut(&c.id).unwrap();
            bounds.extend(200.);
            bounds.intersect(&get_bounds_for_type(c.typ));
        }
    }

    fn handle_blips(&mut self, world: &World) {
        for drone in world.me.drones.values() {
            for (id, blip) in &drone.blips {
                let bounds = self.bounds.get_mut(&id).unwrap();
                bounds.intersect(&get_directional_bounds(*blip, drone.pos));
            }
        }
    }

    pub fn update(&mut self, world: &World) {
        self.initialize(world);
        self.extend_bounds(world);
        self.handle_blips(world);
    }

    pub fn get_bounds(&self, id: i32) -> &Bounds {
        self.bounds.get(&id).unwrap()
    }
}
