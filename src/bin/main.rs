extern crate cgbot;

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

    // game loop
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
