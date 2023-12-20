use std::{default, f32::consts::PI, time::Instant};

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

#[derive(Clone)]
struct GameState {
    drones: [DroneState; 2],
    visited: [[bool; S_CELLS]; 20],
    score: f32,
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
            score: 0.,
        }
    }

    fn visit_cell(&mut self, x: usize, y: usize, score: f32) {
        if !self.visited[x][y] {
            self.visited[x][y] = true;
            self.score += score
        } else {
            self.score += score / 20.;
        }
    }
}

const GENE_SIZE: usize = 20;
const POPULATION_SIZE: usize = 30;
const MUTATIONS_SIZE: usize = 30;
const MUTATIONS_COUNT: usize = 3;
const RANDOM_SIZE: usize = 10;
const CROSSOVER_SIZE: usize = 20;

type Gene = [[Action; 2]; GENE_SIZE];
pub struct Pathfinding<'a> {
    population: Vec<(f32, Gene)>,

    tracker: &'a Tracker,
    exploration_map: &'a ExplorationMap,
    score_map: &'a ScoreMap,
}

impl<'a> Pathfinding<'a> {
    pub fn new(
        tracker: &'a Tracker,
        exploration_map: &'a ExplorationMap,
        score_map: &'a ScoreMap,
    ) -> Self {
        Pathfinding {
            population: Vec::new(),
            tracker,
            exploration_map,
            score_map,
        }
    }

    fn simulate(&self, state: &mut GameState, world: &World, actions: &[Action; 2], iter: usize) {
        for i in 0..2 {
            let drone = &mut state.drones[i];

            if drone.dead {
                state.score -= 1000.;
                continue;
            }

            let action = &actions[i];
            let dir = Vec2::new(1., 0.).rotate(action.angle);
            let mov = dir * 600.;

            drone.pos = (drone.pos + mov).clamp(Vec2::new(0., 0.), Vec2::new(9999., 9999.));

            if action.light && drone.bat >= 5 {
                drone.bat -= 5;
            }

            let iter = iter as f32;

            if drone.pos.y < 400. {
                state.score += (drone.base_scans_cost) as f32 / iter;
                state.score +=
                    (drone.urgent_scans_cost * drone.urgent_scans_cost) as f32 / iter / iter;
                drone.base_scans_cost = 0;
                drone.urgent_scans_cost = 0;
            }

            drone.dead = self
                .tracker
                .monsters
                .values()
                .any(|m| (m.pos - drone.pos).len() < 1200.);

            let (x, y) = position_to_grid_cell(drone.pos, S_CELL_SIZE);
            state.visit_cell(
                x,
                y,
                self.exploration_map.get_score_by_idx(x, y) * self.score_map.get_score_by_idx(x, y),
            );
            let (x, y) = (x as i32, y as i32);

            for dx in -1..2 {
                for dy in -1..2 {
                    if x + dx >= 0
                        && x + dx < S_CELLS as i32
                        && y + dy >= 0
                        && y + dy < S_CELLS as i32
                    {
                        let (x, y) = ((x + dx) as usize, (y + dy) as usize);
                        state.visit_cell(
                            x,
                            y,
                            self.exploration_map.get_score_by_idx(x, y)
                                * self.score_map.get_score_by_idx(x, y),
                        )
                    }
                }
            }
        }
    }

    fn simulate_all(&self, state: &mut GameState, world: &World, gene: &Gene) {
        for (iter, action) in gene.iter().enumerate() {
            self.simulate(state, world, action, iter + 1);
        }
        state.score += (state.drones[0].bat + state.drones[1].bat) as f32 / 30.
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
                action.light = rng.gen_bool(0.5);
            }
        }

        gene
    }

    fn mutation(&mut self, world: &World, state_proto: &GameState) {
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
            self.simulate_all(&mut state, world, &new_gene);
            self.population.push((state.score, new_gene));
        }
    }

    fn crossover(&mut self, world: &World, state_proto: &GameState) {
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
            let mut new_gene11 = gene1.clone();
            let mut new_gene22 = gene2.clone();

            for i in 0..gene1.len() {
                new_gene1[i][0] = gene2[i][0];
                new_gene11[i][0] = gene2[i][1];
                new_gene2[i][0] = gene1[i][0];
                new_gene2[i][1] = gene1[i][1];
            }

            let mut state = state_proto.clone();
            self.simulate_all(&mut state, world, &new_gene1);
            self.population.push((state.score, new_gene1));

            let mut state = state_proto.clone();
            self.simulate_all(&mut state, world, &new_gene2);
            self.population.push((state.score, new_gene2));

            let mut state = state_proto.clone();
            self.simulate_all(&mut state, world, &new_gene11);
            self.population.push((state.score, new_gene11));

            let mut state = state_proto.clone();
            self.simulate_all(&mut state, world, &new_gene22);
            self.population.push((state.score, new_gene22));
        }
    }

    fn add_randoms(&mut self, world: &World, state_proto: &GameState) {
        for i in 0..RANDOM_SIZE {
            let gene = self.random_gene();
            let mut state = state_proto.clone();

            self.simulate_all(&mut state, world, &gene);
            self.population.push((state.score, gene));
        }
    }

    fn selection(&mut self, world: &World, state_proto: &GameState) {
        self.population
            .sort_by_key(|(score, _)| -(score * 100.) as i32);

        self.population.truncate(POPULATION_SIZE);
    }

    pub fn search(&mut self, world: &World) -> [Action; 2] {
        let start = Instant::now();

        let state_proto = GameState::new(world);

        for _ in 0..POPULATION_SIZE {
            let gene = self.random_gene();

            let mut state = state_proto.clone();
            self.simulate_all(&mut state, world, &gene);
            self.population.push((state.score, gene));
        }

        let mut iter = 0;
        while Instant::now().duration_since(start).as_millis() < 40 {
            iter += 1;
            if iter % 5 == 0 {
                self.crossover(world, &state_proto);
            }
            self.add_randoms(world, &state_proto);
            self.mutation(world, &state_proto);
            self.selection(world, &state_proto);
        }

        let (best_score, best_actions) = self.population[0];

        eprintln!("Iterations count: {iter}");
        eprintln!("Best score: {best_score}");

        let esum: f32 = self.exploration_map.map.iter().flatten().sum();
        let ssum: f32 = self.score_map.map.iter().flatten().sum();

        eprintln!("esum: {esum}");
        eprintln!("ssum: {ssum}");

        best_actions[0]
    }
}
