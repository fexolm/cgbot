use super::*;

pub struct Strategy {
    pub bounds_detector: BoundsDetector,
    tracker: Tracker,
    pub exploration_map: ExplorationMap,
    pub score_map: ScoreMap,
    pub pathfinding: Pathfinding,
}

impl Strategy {
    pub fn new() -> Self {
        Strategy {
            bounds_detector: BoundsDetector::new(),
            tracker: Tracker::new(),
            exploration_map: ExplorationMap::new(),
            score_map: ScoreMap::new(),
            pathfinding: Pathfinding::new(),
        }
    }
}

impl Strategy {
    pub fn play(&mut self, world: &World) {
        self.tracker.update(world);
        self.bounds_detector.update(world);
        self.exploration_map.update(world);
        self.score_map.update(world, &self.bounds_detector);

        let actions =
            self.pathfinding
                .search(world, &self.tracker, &self.exploration_map, &self.score_map);

        for (i, drone) in world.me.drones.values().enumerate() {
            let action = actions[i];

            let pos = drone.pos + action.get_move();
            let (x, y) = (pos.x as usize, pos.y as usize);

            let light = if action.get_light() { 1 } else { 0 };

            println!("MOVE {x} {y} {light}");
        }
    }
}
