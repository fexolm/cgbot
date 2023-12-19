pub mod cgbot {
pub mod simulation {
use rand::thread_rng;
use rand::Rng;
use std::collections::HashSet;
use std::time::Instant;
use super::vec2::Vec2;
use super::world::World;
#[derive(Clone, Default)]
pub struct DroneState {
    pos: Vec2,
    scans: HashSet<i32>,
    battery: i32,
}
#[derive(Clone)]
pub struct FishState {
    pos: Vec2,
    id: i32,
}
#[derive(Clone)]
pub struct SimulationState {
    my_drones: Vec<DroneState>,
    enemy_drones: Vec<DroneState>,
    my_scans: HashSet<i32>,
    enemy_scans: HashSet<i32>,
    my_score: i32,
    enemy_score: i32,
    my_fishes_of_type: [i32; 3],
    my_fishes_of_color: [i32; 4],
    enemy_fishes_of_type: [i32; 3],
    enemy_fishes_of_color: [i32; 4],
    visited_cells: [[bool; 20]; 20],
}
impl SimulationState {
    fn new(world: &World) -> Self {
        let mut my_drones = Vec::new();
        let mut enemy_drones = Vec::new();
        let mut my_fishes_of_type = [0; 3];
        let mut my_fishes_of_color = [0; 4];
        let mut enemy_fishes_of_type = [0; 3];
        let mut enemy_fishes_of_color = [0; 4];
        let mut visited_cells = [[false; 20]; 20];
        let mut my_scans = world.me.scans.clone();
        let mut enemy_scans = world.opponent.scans.clone();
        let mut my_score = world.me.score;
        let mut enemy_score = world.opponent.score;
        for i in 0..2 {
            for drone in world.me.drones.values() {
                let mut drone_state = DroneState {
                    pos: drone.pos,
                    battery: drone.bat,
                    scans: drone.scans.clone(),
                };
                my_drones.push(drone_state);
            }
        }
        for drone in world.opponent.drones.values() {
            let mut drone_state = DroneState {
                pos: drone.pos,
                battery: drone.bat,
                scans: drone.scans.clone(),
            };
            enemy_drones.push(drone_state);
        }
        for scan in &world.me.scans {
            if let Some(creature) = world.creatures.get(&scan) {
                my_fishes_of_type[creature.typ as usize] += 1;
                my_fishes_of_color[creature.color as usize] += 1;
            } else {
                eprintln!("222");
            }
        }
        for scan in &world.opponent.scans {
            if let Some(creature) = world.creatures.get(&scan) {
                enemy_fishes_of_type[creature.typ as usize] += 1;
                enemy_fishes_of_color[creature.color as usize] += 1;
            } else {
                eprintln!("333");
            }
        }
        SimulationState {
            my_drones,
            enemy_drones,
            my_fishes_of_type,
            enemy_fishes_of_type,
            my_fishes_of_color,
            enemy_fishes_of_color,
            enemy_scans,
            enemy_score,
            my_scans,
            my_score,
            visited_cells,
        }
    }
    fn evaluate_scans(&mut self, world: &World, idx: usize, mine: bool) {
        if mine {
            if let Some(drone) = self.my_drones.get(idx) {
                for id in drone.scans.iter() {
                    let fish = world.creatures.get(&id).unwrap();
                    let mut points = fish.typ as i32 + 1;
                    if !self.enemy_scans.contains(&id) {
                        points *= 2;
                    }
                    self.my_fishes_of_type[fish.typ as usize] += 1;
                    self.my_fishes_of_color[fish.color as usize] += 1;
                    if self.my_fishes_of_type[fish.typ as usize] == 4 {
                        points += if self.enemy_fishes_of_type[fish.typ as usize] == 4 {
                            4
                        } else {
                            8
                        }
                    }
                    if self.my_fishes_of_color[fish.color as usize] == 3 {
                        points += if self.enemy_fishes_of_color[fish.color as usize] == 3 {
                            3
                        } else {
                            6
                        }
                    }
                    self.my_scans.insert(*id);
                    self.my_score += points
                }
                for scan in &self.my_scans {
                    for i in 0..2 {
                        self.my_drones[i].scans.remove(&scan);
                    }
                }
            } else {
                eprintln!("444");
            }
        } else {
            if let Some(drone) = self.enemy_drones.get(idx) {
                for id in drone.scans.iter() {
                    if let Some(fish) = world.creatures.get(&id) {
                        let mut points = fish.typ as i32 + 1;
                        if !self.my_scans.contains(&id) {
                            points *= 2;
                        }
                        self.enemy_fishes_of_type[fish.typ as usize] += 1;
                        self.enemy_fishes_of_color[fish.color as usize] += 1;
                        if self.enemy_fishes_of_type[fish.typ as usize] == 4 {
                            points += if self.my_fishes_of_type[fish.typ as usize] == 4 {
                                4
                            } else {
                                8
                            }
                        }
                        if self.enemy_fishes_of_color[fish.color as usize] == 3 {
                            points += if self.my_fishes_of_color[fish.color as usize] == 3 {
                                3
                            } else {
                                6
                            }
                        }
                        self.enemy_scans.insert(*id);
                        self.enemy_score += points
                    } else {
                        eprintln!("777");
                    }
                }
                for scan in &self.enemy_scans {
                    for i in 0..2 {
                        self.enemy_drones[i].scans.remove(&scan);
                    }
                }
            } else {
                eprintln!("666");
            }
        }
    }
    fn score(&self, cells: &[[f32; 20]; 20]) -> i32 {
        let mut score = (self.my_score - self.enemy_score) * 100;
        let mut exploration_score = 0.;
        for x in 0..20 {
            for y in 0..20 {
                if self.visited_cells[y][x] {
                    exploration_score += cells[y][x];
                }
            }
        }
        score + exploration_score as i32
    }
}
#[derive(Clone, Copy)]
pub struct Action {
    pub mov: Option<Vec2>,
    pub light: bool,
}
fn evaluate_scans(
    world: &World,
    drone: &mut DroneState,
    scans: &mut HashSet<i32>,
    other_scans: &HashSet<i32>,
) -> i32 {
    let mut res = 0;
    for scan in &drone.scans {}
    drone.scans.clear();
    res
}
pub fn simulate(
    world: &World,
    state: &mut SimulationState,
    my_actions: &[Action; 2],
    enemy_actions: &[Action; 2],
    iters: i32,
) {
    for iter in 0..iters {
        for i in 0..2 {
            let mut pos = state.my_drones[i].pos;
            if let Some(mov) = my_actions[i].mov {
                pos = state.my_drones[i].pos + mov
            } else {
                pos = state.my_drones[i].pos + Vec2 { x: 0., y: -300. }
            }
            pos = state.my_drones[i].pos.clamp(
                Vec2 { x: 0., y: 0. },
                Vec2 {
                    x: 10000.,
                    y: 10000.,
                },
            );
            if pos.y < 500. {
                state.evaluate_scans(world, i, true);
            }
            state.visited_cells[(pos.y / 500.) as usize][(pos.x / 500.) as usize] = true;
            if iter == 0 && my_actions[i].light && state.my_drones[i].battery >= 5 {
                for dx in -1..2 {
                    for dy in -1..2 {
                        let x = (((pos.x / 500.) as i32) + dx).clamp(0, 10);
                        let y = (((pos.y / 500.) as i32) + dy).clamp(0, 10);
                        state.visited_cells[y as usize][x as usize] = true;
                    }
                }
                state.my_drones[i].battery -= 5;
            } else {
                state.my_drones[i].battery += 1;
            }
            state.my_drones[i].pos = pos;
        }
        for i in 0..2 {
            if let Some(mov) = enemy_actions[i].mov {
                state.enemy_drones[i].pos = state.enemy_drones[i].pos + mov
            } else {
                state.my_drones[i].pos = state.my_drones[i].pos + Vec2 { x: 0., y: -300. }
            }
            state.enemy_drones[i].pos = state.enemy_drones[i].pos.clamp(
                Vec2 { x: 0., y: 0. },
                Vec2 {
                    x: 10000.,
                    y: 10000.,
                },
            );
            if state.enemy_drones[i].pos.y < 500. {
                state.evaluate_scans(world, i, false);
            }
        }
    }
}
fn simulate_many(
    world: &World,
    state: &mut SimulationState,
    my_actions: &Vec<[Action; 2]>,
    enemy_actions: &Vec<[Action; 2]>,
    iters: i32,
) {
    for i in 0..my_actions.len() {
        simulate(world, state, &my_actions[i], &enemy_actions[i], iters)
    }
}
fn random_action() -> Action {
    let mut rng = rand::thread_rng();
    let mov = match rng.gen_range(0..8) {
        0 => Vec2 { x: -400., y: -400. },
        1 => Vec2 { x: -600., y: 0. },
        2 => Vec2 { x: -400., y: 400. },
        3 => Vec2 { x: 0., y: -600. },
        4 => Vec2 { x: 0., y: 600. },
        5 => Vec2 { x: 400., y: -400. },
        6 => Vec2 { x: 600., y: 0. },
        7 => Vec2 { x: 400., y: 400. },
        _ => unreachable!(),
    };
    Action {
        mov: Some(mov),
        light: rng.gen_range(0..4) == 1,
    }
}
fn up_action() -> Action {
    let mov = Vec2 { x: 0., y: -800. };
    Action {
        mov: Some(mov),
        light: true,
    }
}
fn generate_random_actions(size: usize) -> Vec<[Action; 2]> {
    let mut res = Vec::with_capacity(size);
    for i in 0..size {
        res.push([random_action(), random_action()]);
    }
    res
}
fn generate_opponent_actions(size: usize) -> Vec<[Action; 2]> {
    let mut res = Vec::with_capacity(size);
    for i in 0..size {
        res.push([up_action(), up_action()]);
    }
    res
}
pub struct Solver {
    prev_best: Vec<[Action; 2]>,
}
impl Solver {
    pub fn new() -> Self {
        let prev_best = generate_random_actions(10);
        Solver { prev_best }
    }
    pub fn find_best_action(&mut self, world: &World, cells: &[[f32; 20]; 20]) -> Vec<Action> {
        let mut best_actions: Vec<[Action; 2]> = self.prev_best.clone();
        best_actions.remove(0);
        best_actions.push([random_action(), random_action()]);
        let opponent_actions = generate_opponent_actions(10);
        let start = Instant::now();
        let initial_state = SimulationState::new(world);
        let mut current_state: SimulationState = initial_state.clone();
        simulate_many(
            world,
            &mut current_state,
            &best_actions,
            &opponent_actions,
            3,
        );
        let mut best_score = current_state.score(cells);
        let mut best_my_score = current_state.my_score;
        let mut iters = 0;
        while Instant::now().duration_since(start).as_millis() < 30 {
            iters += 1;
            let current_actions = generate_random_actions(10);
            let mut current_state: SimulationState = initial_state.clone();
            simulate_many(
                world,
                &mut current_state,
                &current_actions,
                &opponent_actions,
                5,
            );
            let current_score = current_state.score(cells);
            if current_score > best_score {
                best_actions = current_actions;
                best_score = current_score;
            }
        }
        let mut current_state: SimulationState = initial_state.clone();
        simulate_many(
            world,
            &mut current_state,
            &opponent_actions,
            &opponent_actions,
            3,
        );
        eprintln!("Iterations count: {iters}");
        eprintln!("Best score: {best_score}");
        eprintln!("Best my score: {best_my_score}");
        self.prev_best = best_actions.clone();
        vec![best_actions[0][0], best_actions[0][1]]
    }
}
}
pub mod strategy {
use super::simulation;
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
    pub fn len(self) -> f32 {
        ((self.x * self.x) + (self.y * self.y)).sqrt()
    }
    pub fn clamp(self, lt: Vec2, rb: Vec2) -> Vec2 {
        let x = self.x.clamp(lt.x, rb.x);
        let y = self.y.clamp(lt.y, rb.y);
        Vec2 { x, y }
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
fn main() {
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
