use std::collections::HashMap;

use super::*;

#[derive(Clone)]
pub struct Monster {
    id: i32,
    vel: Vec2,
    pub pos: Vec2,
    target: Option<Vec2>,
}

pub struct Tracker {
    pub monsters: Vec<Monster>,
    drone_bat: HashMap<i32, i32>,
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            monsters: Vec::new(),
            drone_bat: HashMap::new(),
        }
    }

    fn get_aggression_radius(&self, drone: &Drone) -> i32 {
        let used_light = if let Some(&old_bat) = self.drone_bat.get(&drone.id) {
            old_bat > drone.bat
        } else {
            false
        };

        if used_light {
            2000
        } else {
            800
        }
    }

    fn update_monsters_targets(&mut self, world: &World) {
        for i in 0..self.monsters.len() {
            let m = &self.monsters[i];
            let target_drone = world
                .me
                .drones
                .iter()
                .chain(world.opponent.drones.iter())
                .filter(|(_, d)| {
                    d.emergency != 1
                        && ((d.pos - m.pos).len() as i32) < self.get_aggression_radius(d)
                })
                .min_by_key(|(_, d)| (d.pos - m.pos).len() as i32);

            let m = &mut self.monsters[i];
            if let Some((_, target_drone)) = target_drone {
                m.target = Some(target_drone.pos)
            } else {
                m.target = None
            }
        }
    }

    fn update_monster_velocities(&mut self, world: &World) {
        let monster_copy = self.monsters.clone();

        for m in &mut self.monsters {
            if let Some(target) = m.target {
                m.vel = (target - m.pos).norm() * 540.
            } else if let Some(closest_monster) = monster_copy
                .iter()
                .filter(|m2| m2.id != m.id && (m2.pos - m.pos).len() < 600.)
                .min_by_key(|m2| (m2.pos - m.pos).len() as i32)
            {
                m.vel = (m.pos - closest_monster.pos).norm() * 200.;
            } else {
                if m.vel.len() > 270. {
                    m.vel = m.vel.norm() * 270.;
                }
            }
        }
    }

    fn update_monster_positions(&mut self) {
        for m in &mut self.monsters {
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

            if let Some(m) = self.monsters.iter_mut().find(|m| m.id == creature.id) {
                m.pos = creature.pos.unwrap();
                m.vel = creature.speed.unwrap();
            } else {
                self.monsters.push(Monster {
                    id: creature.id,
                    vel: creature.speed.unwrap(),
                    pos: creature.pos.unwrap(),
                    target: None,
                });
            }
        }
    }

    fn update_bat(&mut self, world: &World) {
        for (_, d) in world.me.drones.iter().chain(world.opponent.drones.iter()) {
            self.drone_bat.insert(d.id, d.bat);
        }
    }

    pub fn update(&mut self, world: &World) {
        self.update_monster_velocities(world);
        self.update_monster_positions();
        self.update_visible(world);
        self.update_monsters_targets(world);
        self.update_bat(world);

        eprintln!("Monster tracking:");
        for m in &self.monsters {
            eprintln!(
                "id:{}, pos: {}:{} vel: {}:{} target: {}",
                m.id,
                m.pos.x as i32,
                m.pos.y as i32,
                m.vel.x as i32,
                m.vel.y as i32,
                if m.target.is_some() { "yes" } else { "No" }
            );
        }
    }
}
