pub mod cgbot {
pub mod bounds_detector {
use std::collections::HashMap;
use super::world::BlipDirection;
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
    bounds: HashMap<i32, Bounds>,
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
}
pub mod maps {
use super::Creature;
use super::{bounds_detector::BoundsDetector, vec2::Vec2, world::World};
const E_CELLS: usize = 20;
const E_CELL_SIZE: usize = 10000 / E_CELLS;
pub const S_CELLS: usize = 20;
pub const S_CELL_SIZE: usize = 10000 / S_CELLS;
pub struct ExplorationMap {
    pub map: [[f32; E_CELLS]; E_CELLS],
}
pub struct ScoreMap {
    pub map: [[f32; S_CELLS]; S_CELLS],
}
pub fn position_to_grid_cell(pos: Vec2, cell_size: usize) -> (usize, usize) {
    (pos.x as usize / cell_size, pos.y as usize / cell_size)
}
impl ExplorationMap {
    pub fn new() -> Self {
        ExplorationMap {
            map: [[1.; E_CELLS]; E_CELLS],
        }
    }
    const STEP: f32 = 0.01;
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
                if x + dx >= 0 && x + dx < E_CELLS as i32 && y + dy >= 0 && y + dy < E_CELLS as i32
                {
                    self.map[(x + dx) as usize][(y + dy) as usize] = 0.;
                }
            }
        }
    }
    pub fn get_score_by_idx(&self, x: usize, y: usize) -> f32 {
        self.map[x][y]
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
    fn calculate_creature_cost(creature: &Creature, world: &World) -> f32 {
        let mut cost = (creature.typ + 1) as f32;
        if !world.opponent.scans.contains(&creature.id) {
            cost *= 2.;
        }
        let my_type_count = world
            .me
            .scans
            .iter()
            .filter(|id| world.creatures.get(&id).unwrap().typ == creature.typ)
            .count();
        if my_type_count == 4 {
            cost += 4.;
            let opponent_type_count = world
                .opponent
                .scans
                .iter()
                .filter(|id| world.creatures.get(&id).unwrap().typ == creature.typ)
                .count();
            if opponent_type_count < 4 {
                cost += 4.;
            }
        }
        let my_col_count = world
            .me
            .scans
            .iter()
            .filter(|id| world.creatures.get(&id).unwrap().color == creature.color)
            .count();
        if my_col_count == 3 {
            cost += 3.;
            let opponent_col_count = world
                .opponent
                .scans
                .iter()
                .filter(|id| world.creatures.get(&id).unwrap().color == creature.color)
                .count();
            if opponent_col_count < 3 {
                cost += 3.;
            }
        }
        cost
    }
    pub fn update(&mut self, world: &World, bounds_detector: &BoundsDetector) {
        for x in 0..S_CELLS {
            for y in 0..S_CELLS {
                self.map[x][y] = 0.;
            }
        }
        let creatures = world.creatures.values().filter(|c| {
            c.typ != -1
                && !Self::is_in_someone_scan(c.id, world)
                && Self::is_alive(c.id, world)
                && !world.me.scans.contains(&c.id)
        });
        for c in creatures {
            let bounds = bounds_detector.get_bounds(c.id);
            let creature_cost = Self::calculate_creature_cost(c, world);
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
    pub fn get_score_by_idx(&self, x: usize, y: usize) -> f32 {
        self.map[x][y]
    }
}
}
pub mod pathfinding {
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
const GENE_SIZE: usize = 25;
const POPULATION_SIZE: usize = 30;
const MUTATIONS_SIZE: usize = 30;
const MUTATIONS_COUNT: usize = 3;
const RANDOM_SIZE: usize = 10;
const CROSSOVER_SIZE: usize = 20;
type Gene = [[Action; 2]; GENE_SIZE];
struct Simulation<'a> {
    tracker: &'a Tracker,
    exploration_map: &'a ExplorationMap,
    score_map: &'a ScoreMap,
}
pub struct Pathfinding {
    population: Vec<(f32, Gene)>,
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
                action.light = rng.gen_bool(0.5);
            }
        }
        gene
    }
    fn mutation(&mut self, simulation: &Simulation, state_proto: &GameState) {
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
            simulation.simulate_all(&mut state, &new_gene);
            self.population.push((state.score, new_gene));
        }
    }
    fn crossover(&mut self, simulation: &Simulation, state_proto: &GameState) {
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
                new_gene22[i][1] = gene1[i][1];
            }
            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &new_gene1);
            self.population.push((state.score, new_gene1));
            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &new_gene2);
            self.population.push((state.score, new_gene2));
            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &new_gene11);
            self.population.push((state.score, new_gene11));
            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &new_gene22);
            self.population.push((state.score, new_gene22));
        }
    }
    fn add_randoms(&mut self, simulation: &Simulation, state_proto: &GameState) {
        for _ in 0..RANDOM_SIZE {
            let gene = self.random_gene();
            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &gene);
            self.population.push((state.score, gene));
        }
    }
    fn selection(&mut self) {
        self.population
            .sort_by_key(|(score, _)| -(score * 100.) as i32);
        self.population.truncate(POPULATION_SIZE);
    }
    fn modify_prev_generation(&mut self, simulation: &Simulation, state_proto: &GameState) {
        let mut old_population = self.population.clone();
        self.population.clear();
        for (_, action) in &mut old_population {
            let mut gene = Gene::default();
            for i in 1..action.len() {
                gene[i - 1] = action[i];
            }
            gene[gene.len() - 1] = self.random_actions();
            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &gene);
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
        let simulation = Simulation::new(tracker, exploration_map, score_map);
        self.modify_prev_generation(&simulation, &state_proto);
        for _ in 0..POPULATION_SIZE {
            let gene = self.random_gene();
            let mut state = state_proto.clone();
            simulation.simulate_all(&mut state, &gene);
            self.population.push((state.score, gene));
        }
        let mut iter = 0;
        while Instant::now().duration_since(start).as_millis() < 40 {
            iter += 1;
            if iter % 5 == 0 {
                self.crossover(&simulation, &state_proto);
            }
            self.add_randoms(&simulation, &state_proto);
            self.mutation(&simulation, &state_proto);
            self.selection();
        }
        let (best_score, best_actions) = self.population[0];
        eprintln!("Iterations count: {iter}");
        eprintln!("Best score: {best_score}");
        let esum: f32 = simulation.exploration_map.map.iter().flatten().sum();
        let ssum: f32 = simulation.score_map.map.iter().flatten().sum();
        eprintln!("esum: {esum}");
        eprintln!("ssum: {ssum}");
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
        }
    }
    fn simulate(&self, state: &mut GameState, actions: &[Action; 2], iter: usize) {
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
    fn simulate_all(&self, state: &mut GameState, gene: &Gene) {
        for (iter, action) in gene.iter().enumerate() {
            self.simulate(state, action, iter + 1);
        }
        state.score += (state.drones[0].bat + state.drones[1].bat) as f32 / 30.
    }
}
}
pub mod strategy {
use super::*;
pub struct Strategy {
    bounds_detector: BoundsDetector,
    tracker: Tracker,
    exploration_map: ExplorationMap,
    score_map: ScoreMap,
    pathfinding: Pathfinding,
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
}
pub mod tracker {
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
}
pub mod vec2 {
use std::ops::{Add, Mul, Sub};
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, other: f32) -> Vec2 {
        Vec2 {
            x: self.x * other,
            y: self.y * other,
        }
    }
}
impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
    pub fn len(self) -> f32 {
        ((self.x * self.x) + (self.y * self.y)).sqrt()
    }
    pub fn clamp(self, lt: Vec2, rb: Vec2) -> Vec2 {
        let x = self.x.clamp(lt.x, rb.x);
        let y = self.y.clamp(lt.y, rb.y);
        Vec2 { x, y }
    }
    pub fn max(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
    pub fn min(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }
    pub fn rotate(self, angle: f32) -> Vec2 {
        Vec2 {
            x: self.x * angle.cos() - self.y * angle.sin(),
            y: self.x * angle.sin() + self.y * angle.cos(),
        }
    }
}
}
pub mod world {
use super::vec2::Vec2;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Debug,
};
#[derive(Default, Debug)]
pub struct Creature {
    pub id: i32,
    pub color: i8,
    pub typ: i8,
    pub pos: Option<Vec2>,
    pub speed: Option<Vec2>,
}
#[derive(Clone, Copy)]
pub enum BlipDirection {
    TL,
    TR,
    BL,
    BR,
}
impl BlipDirection {
    pub fn from_str(s: &str) -> Self {
        match s {
            "TL" => BlipDirection::TL,
            "TR" => BlipDirection::TR,
            "BL" => BlipDirection::BL,
            "BR" => BlipDirection::BR,
            _ => unreachable!(),
        }
    }
}
impl Default for BlipDirection {
    fn default() -> Self {
        BlipDirection::BL
    }
}
impl Debug for BlipDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TL => write!(f, "TL"),
            Self::TR => write!(f, "TR"),
            Self::BL => write!(f, "BL"),
            Self::BR => write!(f, "BR"),
        }
    }
}
impl Creature {
    pub fn clear(&mut self) {
        self.pos = None;
        self.speed = None;
    }
}
#[derive(Default, Debug)]
pub struct Drone {
    pub id: i32,
    pub pos: Vec2,
    pub bat: i32,
    pub emergency: i32,
    pub blips: HashMap<i32, BlipDirection>,
    pub scans: HashSet<i32>,
}
#[derive(Default, Debug)]
pub struct Player {
    pub score: i32,
    pub scans: HashSet<i32>,
    pub drones: BTreeMap<i32, Drone>,
}
impl Player {
    pub fn clear(&mut self) {
        self.scans.clear();
        self.drones.clear();
    }
}
#[derive(Default, Debug)]
pub struct World {
    pub creatures: HashMap<i32, Creature>,
    pub me: Player,
    pub opponent: Player,
}
impl World {
    pub fn clear(&mut self) {
        self.me.clear();
        self.opponent.clear();
    }
}
}
pub use bounds_detector::*;
pub use maps::*;
pub use pathfinding::*;
pub use strategy::*;
pub use tracker::*;
pub use vec2::*;
pub use world::*;
}
use std::{collections::HashMap, io};
use cgbot::{
    strategy::Strategy,
    vec2::Vec2,
    world::{BlipDirection, Creature, Drone, World},
};
macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}
#[cfg(debug_assertions)]
fn check_debug() {
    eprintln!("Debugging enabled");
}
#[cfg(not(debug_assertions))]
fn check_debug() {
    eprintln!("Debugging disabled");
}
fn main() {
    check_debug();
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let creature_count = parse_input!(input_line, usize);
    let mut creatures = HashMap::with_capacity(creature_count);
    for _ in 0..creature_count as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let id = parse_input!(inputs[0], i32);
        let color = parse_input!(inputs[1], i8);
        let typ = parse_input!(inputs[2], i8);
        creatures.insert(
            id,
            Creature {
                id,
                color,
                typ,
                ..Default::default()
            },
        );
    }
    let mut world = World {
        creatures,
        ..Default::default()
    };
    let mut strategy = Strategy::new();
    loop {
        world.clear();
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        world.me.score = parse_input!(input_line, i32);
        input_line.clear();
        io::stdin().read_line(&mut input_line).unwrap();
        world.opponent.score = parse_input!(input_line, i32);
        input_line.clear();
        io::stdin().read_line(&mut input_line).unwrap();
        let my_scan_count = parse_input!(input_line, usize);
        world.me.scans.reserve(my_scan_count);
        input_line.clear();
        for _ in 0..my_scan_count as usize {
            io::stdin().read_line(&mut input_line).unwrap();
            let creature_id = parse_input!(input_line, i32);
            world.me.scans.insert(creature_id);
            input_line.clear();
        }
        io::stdin().read_line(&mut input_line).unwrap();
        let foe_scan_count = parse_input!(input_line, usize);
        world.opponent.scans.reserve(foe_scan_count);
        input_line.clear();
        for _ in 0..foe_scan_count as usize {
            io::stdin().read_line(&mut input_line).unwrap();
            let creature_id = parse_input!(input_line, i32);
            world.opponent.scans.insert(creature_id);
            input_line.clear();
        }
        io::stdin().read_line(&mut input_line).unwrap();
        let my_drone_count = parse_input!(input_line, usize);
        input_line.clear();
        for _ in 0..my_drone_count as usize {
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let id = parse_input!(inputs[0], i32);
            let x = parse_input!(inputs[1], i32) as f32;
            let y = parse_input!(inputs[2], i32) as f32;
            let emergency = parse_input!(inputs[3], i32);
            let bat = parse_input!(inputs[4], i32);
            world.me.drones.insert(
                id,
                Drone {
                    id,
                    pos: Vec2 { x, y },
                    bat,
                    emergency,
                    ..Default::default()
                },
            );
            input_line.clear();
        }
        io::stdin().read_line(&mut input_line).unwrap();
        let foe_drone_count = parse_input!(input_line, usize);
        input_line.clear();
        for _ in 0..foe_drone_count as usize {
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let id = parse_input!(inputs[0], i32);
            let x = parse_input!(inputs[1], i32) as f32;
            let y = parse_input!(inputs[2], i32) as f32;
            let emergency = parse_input!(inputs[3], i32);
            let bat = parse_input!(inputs[4], i32);
            world.opponent.drones.insert(
                id,
                Drone {
                    id,
                    pos: Vec2 { x, y },
                    bat,
                    emergency,
                    ..Default::default()
                },
            );
            input_line.clear();
        }
        io::stdin().read_line(&mut input_line).unwrap();
        let drone_scan_count = parse_input!(input_line, i32);
        input_line.clear();
        for _ in 0..drone_scan_count as usize {
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let creature_id = parse_input!(inputs[1], i32);
            if let Some(drone) = world.me.drones.get_mut(&drone_id) {
                drone.scans.insert(creature_id);
            } else if let Some(drone) = world.opponent.drones.get_mut(&drone_id) {
                drone.scans.insert(creature_id);
            } else {
                unreachable!()
            }
            input_line.clear();
        }
        io::stdin().read_line(&mut input_line).unwrap();
        let visible_creature_count = parse_input!(input_line, i32);
        input_line.clear();
        for creature in world.creatures.values_mut() {
            creature.clear();
        }
        for _ in 0..visible_creature_count as usize {
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let id = parse_input!(inputs[0], i32);
            let x = parse_input!(inputs[1], i32) as f32;
            let y = parse_input!(inputs[2], i32) as f32;
            let vx = parse_input!(inputs[3], i32) as f32;
            let vy = parse_input!(inputs[4], i32) as f32;
            if let Some(creature) = world.creatures.get_mut(&id) {
                creature.pos = Some(Vec2 { x, y });
                creature.speed = Some(Vec2 { x: vx, y: vy });
            }
            input_line.clear();
        }
        io::stdin().read_line(&mut input_line).unwrap();
        let radar_blip_count = parse_input!(input_line, i32);
        input_line.clear();
        for _ in 0..radar_blip_count as usize {
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let drone_id = parse_input!(inputs[0], i32);
            let creature_id = parse_input!(inputs[1], i32);
            let radar = inputs[2].trim().to_string();
            if let Some(drone) = world.me.drones.get_mut(&drone_id) {
                drone
                    .blips
                    .insert(creature_id, BlipDirection::from_str(&radar));
            }
            input_line.clear();
        }
        strategy.play(&world);
    }
}
