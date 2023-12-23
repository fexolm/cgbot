use std::{f32::consts::PI, time::Instant};

use rand::Rng;

use super::*;

#[derive(Default, Clone, Copy)]
pub struct Action {
    angle: f32,
    light: bool,
}

impl Action {
    pub fn get_light(&self) -> bool {
        self.light
    }

    pub fn get_move(&self) -> Vec2 {
        Vec2::new(1., 0.).rotate(self.angle) * 600.
    }
}

#[derive(Default, Clone, Copy)]
struct DroneState {
    pos: Vec2,
    bat: i32,
    dead: bool,
    base_scans_cost: i32,
    urgent_scans_cost: i32,
}

#[derive(Clone, Copy, Default)]
pub struct Score {
    saving_scans_score: f32,
    saving_urgent_scans_score: f32,
    exploration_score: f32,
    dead_score: f32,
}

impl Score {
    fn value(&self) -> f32 {
        self.saving_scans_score
            + self.saving_urgent_scans_score
            + self.exploration_score
            + self.dead_score
    }
}

#[derive(Clone)]
struct GameState {
    drones: [DroneState; 2],
    visited: [[bool; S_CELLS]; S_CELLS],
    score: Score,
    iter: i32,
}

fn estimate_drones_scans_profit(world: &World, drone: &Drone, state: &mut DroneState) {
    let mut by_typ_count = [0; 3];
    let mut by_color_count = [0; 4];

    for id in &world.me.scans {
        let creature = world.creatures.get(&id).unwrap();
        by_typ_count[creature.typ as usize] += 1;
        by_color_count[creature.color as usize] += 1;
    }

    for id in &drone.scans {
        let creature = world.creatures.get(&id).unwrap();
        let fish_cost = (creature.typ + 1) as i32;
        state.base_scans_cost += fish_cost;

        if !world.opponent.scans.contains(&id) {
            state.urgent_scans_cost += fish_cost;
        }

        by_typ_count[creature.typ as usize] += 1;
        by_color_count[creature.color as usize] += 1;

        if by_typ_count[creature.typ as usize] == 4 {
            state.base_scans_cost += 4;

            let opponents_count = world
                .opponent
                .scans
                .iter()
                .filter(|id| world.creatures.get(&id).unwrap().typ == creature.typ)
                .count();
            if opponents_count < 4 {
                state.urgent_scans_cost += 4;
            }
        }

        if by_color_count[creature.color as usize] == 3 {
            state.base_scans_cost += 3;

            let opponents_count = world
                .opponent
                .scans
                .iter()
                .filter(|id| world.creatures.get(&id).unwrap().color == creature.color)
                .count();
            if opponents_count < 3 {
                state.urgent_scans_cost += 3;
            }
        }
    }
}

impl GameState {
    fn new(world: &World) -> Self {
        let mut drones = [DroneState::default(); 2];
        for (i, drone) in world.me.drones.values().enumerate() {
            drones[i].pos = drone.pos;
            drones[i].bat = drone.bat;
            drones[i].dead = drone.emergency == 1;
            estimate_drones_scans_profit(world, drone, &mut drones[i]);
        }
        let visited = [[false; S_CELLS]; 20];

        GameState {
            drones,
            visited,
            score: Score::default(),
            iter: world.iter,
        }
    }

    fn visit_score(&self, x: usize, y: usize) -> f32 {
        if !self.visited[x][y] {
            1.
        } else {
            1. / 100.
        }
    }

    fn visit_cell(&mut self, x: usize, y: usize) {
        self.visited[x][y] = true;
    }
}

const GENE_SIZE: usize = 25;
const POPULATION_SIZE: usize = 30;
const MUTATIONS_SIZE: usize = 30;
const MUTATIONS_COUNT: usize = 3;
const RANDOM_SIZE: usize = 10;
const CROSSOVER_SIZE: usize = 30;

type Gene = [[Action; 2]; GENE_SIZE];
struct Simulation<'a> {
    tracker: &'a Tracker,
    exploration_map: &'a ExplorationMap,
    score_map: &'a ScoreMap,
    dead_simulations: i32,
    total_simulations: i32,
}

pub struct Pathfinding {
    pub population: Vec<(Score, Gene)>,
}

impl Pathfinding {
    pub fn new() -> Self {
        Pathfinding {
            population: Vec::new(),
        }
    }

    fn random_actions(&self) -> [Action; 2] {
        let mut rng = rand::thread_rng();
        let mut actions: [Action; 2] = Default::default();

        for action in &mut actions {
            action.light = rng.gen_bool(0.5);
            action.angle = rng.gen_range(-PI..PI);
        }

        actions
    }

    fn random_gene(&self) -> Gene {
        let mut rng = rand::thread_rng();
        let mut gene = Gene::default();

        for actions in &mut gene {
            for action in actions {
                action.angle = rng.gen_range(-PI..PI);
                action.light = false;
            }
        }

        gene
    }

    fn straight_top_gene(&self) -> Gene {
        let mut gene = Gene::default();

        for actions in &mut gene {
            for action in actions {
                action.angle = -PI / 2.;
                action.light = false;
            }
        }

        gene
    }

    fn add_straight_top(&mut self, simulation: &mut Simulation, state_proto: &GameState) {
        let mut gene = self.straight_top_gene();
        let mut state = state_proto.clone();
        simulation.simulate_all(&mut state, &mut gene);
        self.population.push((state.score, gene));
    }

    fn mutation(&mut self, simulation: &mut Simulation, state_proto: &GameState) {
        let mut rng = rand::thread_rng();

        let population_size = self.population.len();

        for _ in 0..MUTATIONS_SIZE {
            let idx = rng.gen_range(0..population_size);

            let (_, mut new_gene) = self.population[idx].clone();

            for _ in 0..MUTATIONS_COUNT {
                let mut_idx = rng.gen_range(0..new_gene.len());
                new_gene[mut_idx] = self.random_actions();
            }

            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &mut new_gene);
            self.population.push((state.score, new_gene));
        }
    }

    fn crossover(&mut self, simulation: &mut Simulation, state_proto: &GameState) {
        let mut rng = rand::thread_rng();

        let population_size = self.population.len();

        for _ in 0..CROSSOVER_SIZE {
            let idx1 = rng.gen_range(0..population_size);
            let mut idx2 = rng.gen_range(0..population_size);

            while idx2 == idx1 {
                idx2 = rng.gen_range(0..population_size);
            }

            let (_, gene1) = &self.population[idx1];
            let (_, gene2) = &self.population[idx2];

            let mut new_gene1 = gene1.clone();
            let mut new_gene2 = gene2.clone();

            for i in 0..gene1.len() {
                new_gene1[i][1] = gene2[i][1];
                new_gene2[i][1] = gene1[i][1];
            }

            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &mut new_gene1);
            self.population.push((state.score, new_gene1));

            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &mut new_gene2);
            self.population.push((state.score, new_gene2));
        }
    }

    fn add_randoms(&mut self, simulation: &mut Simulation, state_proto: &GameState) {
        for _ in 0..RANDOM_SIZE {
            let mut gene = self.random_gene();
            let mut state = state_proto.clone();

            simulation.simulate_all(&mut state, &mut gene);
            self.population.push((state.score, gene));
        }
    }

    fn selection(&mut self) {
        self.population
            .sort_by_key(|(score, _)| -(score.value() * 100.) as i32);

        self.population.truncate(POPULATION_SIZE);
    }

    fn modify_prev_generation(&mut self, simulation: &mut Simulation, state_proto: &GameState) {
        let mut old_population = self.population.clone();
        self.population.clear();

        for (_, action) in &mut old_population {
            let mut gene = Gene::default();

            for i in 1..action.len() {
                gene[i - 1] = action[i];
            }

            gene[gene.len() - 1] = self.random_actions();

            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &mut gene);
            self.population.push((state.score, gene));
        }
    }

    pub fn search(
        &mut self,
        world: &World,
        tracker: &Tracker,
        exploration_map: &ExplorationMap,
        score_map: &ScoreMap,
    ) -> [Action; 2] {
        let start = Instant::now();

        let state_proto = GameState::new(world);

        let mut simulation = Simulation::new(tracker, exploration_map, score_map);

        self.modify_prev_generation(&mut simulation, &state_proto);
        self.add_straight_top(&mut simulation, &state_proto);

        for _ in 0..POPULATION_SIZE {
            let mut gene = self.random_gene();

            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &mut gene);
            self.population.push((state.score, gene));
        }

        let mut iter = 0;
        while Instant::now().duration_since(start).as_millis() < 40 {
            iter += 1;
            self.add_randoms(&mut simulation, &state_proto);
            self.mutation(&mut simulation, &state_proto);
            self.crossover(&mut simulation, &state_proto);
            self.selection();
        }

        let (best_score, best_actions) = self.population[0];

        eprintln!("Iterations count: {iter}");
        eprintln!("Best score: {}", best_score.value());
        eprintln!("Exploration score: {}", best_score.exploration_score);
        eprintln!("Saving scans score: {}", best_score.saving_scans_score);
        eprintln!(
            "Saving urgent scans score: {}",
            best_score.saving_urgent_scans_score
        );
        eprintln!("Dead score: {}", best_score.dead_score);
        eprintln!("Dead simulations: {}", simulation.dead_simulations);
        eprintln!("Total simulations: {}", simulation.total_simulations);

        best_actions[0]
    }
}

impl<'a> Simulation<'a> {
    pub fn new(
        tracker: &'a Tracker,
        exploration_map: &'a ExplorationMap,
        score_map: &'a ScoreMap,
    ) -> Self {
        Simulation {
            tracker,
            exploration_map,
            score_map,
            dead_simulations: 0,
            total_simulations: 0,
        }
    }

    fn adjust_drone_move(&self, state: &mut GameState, drone_idx: usize, action: &mut Action) {
        let drone = &mut state.drones[drone_idx];

        let base_angle = action.angle;

        for rotation in [(-PI / 3.), (PI / 3.), (2. * PI / 3.), (-2. * PI / 3.)] {
            let dir = Vec2::new(1., 0.).rotate(action.angle);
            let mov = dir * 600.;
            let new_pos = (drone.pos + mov).clamp(Vec2::new(0., 0.), Vec2::new(9999., 9999.));

            drone.dead = self
                .tracker
                .monsters
                .values()
                .any(|m| (m.pos - new_pos).len() < 1200.);

            if !drone.dead {
                return;
            }

            action.angle = base_angle + rotation;
            if action.angle > PI {
                action.angle -= 2. * PI;
            }
        }
    }

    fn simulate(&mut self, state: &mut GameState, actions: &mut [Action; 2], iter: usize) {
        for i in 0..2 {
            let action = &mut actions[i];

            if state.drones[i].dead {
                state.score.dead_score -= 1000.;
                continue;
            }

            self.adjust_drone_move(state, i, action);

            if state.drones[i].dead {
                state.score.dead_score -= 1000.;
                continue;
            }

            let dir = Vec2::new(1., 0.).rotate(action.angle);
            let mov = dir * 300.;

            for _ in 0..2 {
                let drone = &mut state.drones[i];

                drone.pos = (drone.pos + mov).clamp(Vec2::new(0., 0.), Vec2::new(9999., 9999.));

                let (x, y) = position_to_grid_cell(drone.pos, S_CELL_SIZE);

                state.score.exploration_score += self.exploration_map.get_score_by_idx(x, y)
                    * self.score_map.get_score_by_idx(x, y)
                    * state.visit_score(x, y);
                state.visit_cell(x, y);
            }

            let drone = &mut state.drones[i];

            if drone.pos.y < 400. {
                let iter = iter as f32;

                state.score.saving_scans_score += (drone.base_scans_cost) as f32 / iter;
                state.score.saving_urgent_scans_score +=
                    (drone.urgent_scans_cost * drone.urgent_scans_cost) as f32 / iter / iter;
                drone.base_scans_cost = 0;
                drone.urgent_scans_cost = 0;
            }

            let drone = &state.drones[i];

            let (x, y) = position_to_grid_cell(drone.pos, S_CELL_SIZE);
            let (x, y) = (x as i32, y as i32);

            let mut light_score = 0.;

            if drone.bat >= 5 {
                for dx in -1..2 {
                    for dy in -1..2 {
                        if x + dx >= 0
                            && x + dx < S_CELLS as i32
                            && y + dy >= 0
                            && y + dy < S_CELLS as i32
                        {
                            let (x, y) = ((x + dx) as usize, (y + dy) as usize);

                            light_score += self.exploration_map.get_score_by_idx(x, y)
                                * self.score_map.get_score_by_idx(x, y)
                                * state.visit_score(x, y);
                        }
                    }
                }
                if light_score > 0.5 {
                    action.light = true;
                } else {
                    action.light = false;
                }
            } else {
                action.light = false;
            }

            if action.light {
                for dx in -1..2 {
                    for dy in -1..2 {
                        if x + dx >= 0
                            && x + dx < S_CELLS as i32
                            && y + dy >= 0
                            && y + dy < S_CELLS as i32
                        {
                            let (x, y) = ((x + dx) as usize, (y + dy) as usize);
                            state.visit_cell(x, y);
                        }
                    }
                }

                state.score.exploration_score += light_score;
            }

            let drone = &mut state.drones[i];

            if action.light {
                drone.bat -= 5;
            } else {
                drone.bat += 1;
            }
        }
    }

    fn simulate_all(&mut self, state: &mut GameState, gene: &mut Gene) {
        for (iter, action) in gene.iter_mut().enumerate() {
            self.simulate(state, action, iter + 1);
        }

        self.total_simulations += 1;

        if state.drones[0].dead || state.drones[1].dead {
            self.dead_simulations += 1;
        }
    }
}
