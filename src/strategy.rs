use std::collections::HashSet;

use super::*;

pub struct Strategy {
    bounds_detector: BoundsDetector,
    tracker: Tracker,
    exploration_map: ExplorationMap,
    score_map: ScoreMap,
}

impl Strategy {
    pub fn new() -> Self {
        Strategy {
            bounds_detector: BoundsDetector::new(),
            tracker: Tracker::new(),
            exploration_map: ExplorationMap::new(),
            score_map: ScoreMap::new(),
        }
    }
}

impl Strategy {
    fn is_in_someone_scan(&self, id: i32, world: &World) -> bool {
        world
            .me
            .drones
            .values()
            .into_iter()
            .any(|drone| drone.scans.contains(&id))
    }

    fn is_alive(&self, id: i32, world: &World) -> bool {
        world
            .me
            .drones
            .values()
            .any(|drone| drone.blips.contains_key(&id))
    }

    fn find_nearest_target_pos(&self, world: &World, drone: &Drone) -> Option<Vec2> {
        let other_drone = world.me.drones.values().find(|d| d.id != drone.id).unwrap();

        if let Some(target) = world
            .creatures
            .values()
            .filter(|c| {
                !world.me.scans.contains(&c.id)
                    && !self.is_in_someone_scan(c.id, world)
                    && self.is_alive(c.id, world)
                    && c.typ != -1
            })
            .min_by_key(|c| {
                let bounds_center = self.bounds_detector.get_bounds(c.id).get_center();
                let dist_to_center = (bounds_center - drone.pos).len();
                let dist_to_other_drone = (bounds_center - other_drone.pos).len();

                (dist_to_center - (dist_to_other_drone / 3.)) as i32
            })
        {
            let bounds = self.bounds_detector.get_bounds(target.id);

            eprintln!("Moving to target {} with bounds: {:#?}", target.id, bounds);

            Some(bounds.get_center())
        } else {
            for c in world.creatures.values() {
                if world.me.scans.contains(&c.id) {
                    eprintln!("{} is in my scans...", c.id);
                    continue;
                }
                if self.is_in_someone_scan(c.id, world) {
                    eprintln!("{} is in drone scans...", c.id);
                    continue;
                }
                if c.typ == -1 {
                    eprintln!("{} is a monster...", c.id);
                    continue;
                }
                eprintln!("{} is OK!!", c.id);
            }

            None
        }
    }

    fn get_total_live_fishes_count(world: &World) -> i32 {
        world
            .me
            .drones
            .values()
            .flat_map(|d| &d.blips)
            .map(|(&id, _)| id)
            .filter(|&id| !world.me.scans.contains(&id))
            .collect::<HashSet<i32>>()
            .len() as i32
    }

    fn get_total_scaned_fishes_count(world: &World) -> i32 {
        world
            .me
            .drones
            .values()
            .flat_map(|d| &d.scans)
            .copied()
            .collect::<HashSet<i32>>()
            .len() as i32
    }

    fn find_monster_nearby(&self, drone: &Drone) -> Option<Vec2> {
        self.tracker
            .monsters
            .values()
            .map(|m| m.pos)
            .min_by_key(|&pos| (pos - drone.pos).len() as i32)
    }

    pub fn play(&mut self, world: &World) {
        self.tracker.update(world);
        self.bounds_detector.update(world);
        self.exploration_map.update(world);
        self.score_map.update(world, &self.bounds_detector);

        for drone in world.me.drones.values() {
            if let Some(monster_pos) = self.find_monster_nearby(drone) {
                eprintln!("found monster!!");

                if (monster_pos - drone.pos).len() < 2000. {
                    eprintln!("avoiding...");

                    let get_out_vec = ((drone.pos - monster_pos) * 100.)
                        .clamp(Vec2::new(0., 0.), Vec2::new(10000., 10000.));
                    let (x, y) = (get_out_vec.x as i32, get_out_vec.y as i32);
                    println!("MOVE {x} {y} 0 AAAAAAA.....");
                    continue;
                }
            }

            if Self::get_total_live_fishes_count(world)
                <= Self::get_total_scaned_fishes_count(world)
            {
                println!("MOVE {} 0 0", drone.pos.x);
                continue;
            }

            if let Some(pos) = self.find_nearest_target_pos(world, drone) {
                let light = if (pos - drone.pos).len() < 2000. && drone.bat > 5 {
                    1
                } else {
                    0
                };

                if light == 1 {
                    self.exploration_map.use_light(drone.pos);
                }

                let (x, y) = (pos.x as i32, pos.y as i32);
                println!("MOVE {x} {y} {light}");
                continue;
            }

            eprintln!("Nothing to do: moving upwards");
            println!("MOVE {} 0 0", drone.pos.x);
        }
    }
}
