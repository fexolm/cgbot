use std::collections::HashMap;

use super::*;

pub struct MetaStrategy {
    fish_cost: HashMap<i32, f32>,
}

impl MetaStrategy {
    pub fn new() -> Self {
        MetaStrategy {
            fish_cost: HashMap::new(),
        }
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

    pub fn update(&mut self, world: &World) {
        let fishes = world.creatures.values().filter(|c| c.typ != -1);

        for f in fishes {
            self.fish_cost
                .insert(f.id, Self::calculate_creature_cost(f, world));
        }
    }

    pub fn get_fish_cost(&self, id: i32) -> f32 {
        *self.fish_cost.get(&id).unwrap()
    }
}
