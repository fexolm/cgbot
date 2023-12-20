use std::collections::HashMap;

use super::vec2::Vec2;
use super::world::World;

pub struct Monster {
    pub pos: Vec2,
    pub vel: Vec2,
}

pub struct Tracker {
    pub monsters: HashMap<i32, Monster>,
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            monsters: HashMap::new(),
        }
    }

    fn update_positions(&mut self) {
        for m in self.monsters.values_mut() {
            m.pos = m.pos + m.vel;

            if m.pos.x < 0.0 || m.pos.x > 10000. {
                m.vel.x = -m.vel.x;
            }

            if m.pos.y < 2500. || m.pos.y > 10000. {
                m.vel.y = -m.vel.y;
            }

            m.pos = m.pos.clamp(Vec2::new(0., 2500.), Vec2::new(10000., 10000.));
        }
    }

    fn update_visible(&mut self, world: &World) {
        for creature in world.creatures.values() {
            if creature.typ != -1 || creature.pos.is_none() {
                continue;
            }

            self.monsters.insert(
                creature.id,
                Monster {
                    pos: creature.pos.unwrap(),
                    vel: creature.speed.unwrap(),
                },
            );
        }
    }

    pub fn update(&mut self, world: &World) {
        self.update_positions();
        self.update_visible(world);
    }
}
