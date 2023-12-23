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

impl Creature {
    pub fn new(id: i32, color: i8, typ: i8, pos: Option<Vec2>, speed: Option<Vec2>) -> Self {
        Creature {
            id,
            color,
            typ,
            pos,
            speed,
        }
    }
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

#[derive(Default, Debug, Clone)]
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
    pub iter: i32,
}

impl World {
    pub fn clear(&mut self) {
        self.me.clear();
        self.opponent.clear();
    }
}
