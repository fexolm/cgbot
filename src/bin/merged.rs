pub mod cgbot {
pub mod bounds_detector {
use std::collections::HashMap;
use super::world::BlipDirection;
use super::vec2::Vec2;
use super::world::World;
#[derive(Debug)]
pub struct Bounds {
    top_left: Vec2,
    bot_right: Vec2,
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
pub mod strategy {
use std::collections::HashSet;
use super::{bounds_detector::BoundsDetector, tracker::*, vec2::Vec2, world::*};
pub struct Strategy {
    bounds_detector: BoundsDetector,
    tracker: Tracker,
}
impl Strategy {
    pub fn new() -> Self {
        Strategy {
            bounds_detector: BoundsDetector::new(),
            tracker: Tracker::new(),
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
    fn get_total_live_fishes_count(&self, world: &World) -> i32 {
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
    fn get_total_scaned_fishes_count(&self, world: &World) -> i32 {
        world
            .me
            .drones
            .values()
            .flat_map(|d| &d.scans)
            .copied()
            .collect::<HashSet<i32>>()
            .len() as i32
    }
    fn find_monster_nearby(&self, drone: &Drone, world: &World) -> Option<Vec2> {
        self.tracker
            .monsters
            .values()
            .map(|m| m.pos)
            .min_by_key(|&pos| (pos - drone.pos).len() as i32)
    }
    pub fn play(&mut self, world: &World) {
        self.tracker.update(world);
        self.bounds_detector.update(world);
        for drone in world.me.drones.values() {
            if let Some(monster_pos) = self.find_monster_nearby(drone, world) {
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
            if self.get_total_live_fishes_count(world) <= self.get_total_scaned_fishes_count(world)
            {
                println!("MOVE {} 0 0", drone.pos.x);
                continue;
            }
            if let Some(pos) = self.find_nearest_target_pos(world, drone) {
                let light = 0;
                let (x, y) = (pos.x as i32, pos.y as i32);
                println!("MOVE {x} {y} {light}");
                continue;
            }
            eprintln!("Nothing to do: moving upwards");
            println!("MOVE {} 0 0", drone.pos.x);
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
