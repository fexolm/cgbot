extern crate cgbot;

use std::collections::{HashMap, HashSet};

use canvas::curves::Bounds;
use cgbot::*;

use flo_canvas::*;
use flo_draw::*;

use futures::executor;
use futures::prelude::*;

use rand::Rng;
struct SimWorld {
    creatures: Vec<(Creature, bool)>,
    drones: Vec<Drone>,
}

impl SimWorld {
    fn gen_random_world() -> Self {
        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

        let mut creatures = Vec::new();

        for typ in 0..3 {
            for col in 0..4 {
                let min_depth = (typ + 1) * 2500;
                let max_depth = min_depth + 2500;

                let x = rng.gen_range(0..10000);
                let y = rng.gen_range(min_depth..max_depth);
                let visible = rng.gen_bool(0.3);

                let creature = Creature::new(
                    col * 3 + typ,
                    col as i8,
                    typ as i8,
                    Some(Vec2::new(x as f32, y as f32)),
                    Some(Vec2::new(0., 0.)),
                );

                creatures.push((creature, visible));
            }
        }

        for i in 0..3 {
            let x = rng.gen_range(0..10000);
            let y = rng.gen_range(2500..10000);
            let visible = rng.gen_bool(1.);

            let creature = Creature::new(
                12 + i,
                -1,
                -1,
                Some(Vec2::new(x as f32, y as f32)),
                Some(Vec2::new(0., 0.)),
            );

            creatures.push((creature, visible));
        }

        let mut drones = Vec::new();

        for i in 0..2 {
            let x = rng.gen_range(0..10000);
            let y = rng.gen_range(0..10000);

            let mut blips = HashMap::new();

            for i in 0..15 {
                let (creature, _) = &creatures[i];

                let dir = match (
                    creature.pos.unwrap().x < x as f32,
                    creature.pos.unwrap().y < y as f32,
                ) {
                    (true, true) => BlipDirection::TL,
                    (true, false) => BlipDirection::BL,
                    (false, true) => BlipDirection::TR,
                    (false, false) => BlipDirection::BR,
                };

                blips.insert(creature.id, dir);
            }

            let drone = Drone {
                id: i + 15,
                pos: Vec2::new(x as f32, y as f32),
                bat: 30,
                emergency: 0,
                blips,
                scans: HashSet::new(),
            };
            drones.push(drone);
        }

        SimWorld { creatures, drones }
    }

    fn build_world(&self) -> World {
        let creatures = self
            .creatures
            .iter()
            .map(|(c, vis)| {
                (
                    c.id,
                    Creature::new(
                        c.id,
                        c.color,
                        c.typ,
                        if *vis { c.pos } else { None },
                        if *vis { c.speed } else { None },
                    ),
                )
            })
            .collect();

        let mut me = Player::default();
        me.drones = self.drones.iter().map(|d| (d.id, d.clone())).collect();

        let mut opponent = Player::default();
        opponent.drones = self.drones.iter().map(|d| (d.id, d.clone())).collect();

        World {
            creatures,
            me,
            opponent,
        }
    }
}

fn draw_circle_at_pos(gc: &mut CanvasGraphicsContext, pos: Vec2, col: Color) {
    let pos = pos * 0.1;
    gc.new_path();

    gc.circle(pos.x, 1000. - pos.y, 25.);

    gc.fill_color(col);

    gc.fill();
    gc.line_width(1.0);
    gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
    gc.stroke();
}

fn draw_bounds(gc: &mut CanvasGraphicsContext, tl: Vec2, br: Vec2) {
    let mut rng = rand::thread_rng();
    let tl = tl * 0.1;
    let br = br * 0.1;

    gc.new_path();

    gc.rect(tl.x, 1000. - tl.y, br.x, 1000. - br.y);
    gc.fill_color(Color::Rgba(
        rng.gen_range(0. ..1.),
        rng.gen_range(0. ..1.),
        rng.gen_range(0. ..1.),
        0.1,
    ));
    gc.fill();
    gc.line_width(1.0);
    gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
    gc.stroke();
}

fn draw_world(gc: &mut CanvasGraphicsContext, world: &SimWorld) {
    for (c, vis) in &world.creatures {
        let pos = c.pos.unwrap();

        let opacity = if *vis { 1. } else { 0.5 };

        let color = if c.typ == -1 {
            Color::Rgba(1., 0., 0., opacity)
        } else {
            Color::Rgba(0., 1., 0., opacity)
        };

        draw_circle_at_pos(gc, pos, color);
    }

    for d in &world.drones {
        draw_circle_at_pos(gc, d.pos, Color::Rgba(0., 0., 1., 1.));
    }
}

fn draw_lines(gc: &mut CanvasGraphicsContext) {
    for y in [250., 500., 750.] {
        gc.new_path();

        gc.move_to(0., y);
        gc.line_to(1000., y);

        gc.rect(0., 0., 1000., 1000.);

        gc.line_width(1.0);
        gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 0.3));
        gc.stroke();
    }
}

fn draw_weights(
    gc: &mut CanvasGraphicsContext,
    score_map: &ScoreMap,
    exploration_map: &ExplorationMap,
) {
    let mut max: f32 = 0.;
    for x in 0..S_CELLS {
        for y in 0..S_CELLS {
            let score = score_map.get_score_by_idx(x, y);
            let exploration = exploration_map.get_score_by_idx(x, y);

            let weight = score * exploration as f32;

            max = max.max(weight);
        }
    }

    for x in 0..S_CELLS {
        for y in 0..S_CELLS {
            let score = score_map.get_score_by_idx(x, y);
            let exploration = exploration_map.get_score_by_idx(x, y);

            let weight = score * exploration;

            gc.new_path();
            gc.rect(
                50. * x as f32,
                1000. - 50. * y as f32,
                50. * (x + 1) as f32,
                1000. - 50. * (y + 1) as f32,
            );

            gc.fill_color(Color::Rgba(1. - weight / max, 1., 1. - weight / max, 0.6));

            gc.fill();
        }
    }
}

fn draw_paths(
    gc: &mut CanvasGraphicsContext,
    start_positions: [Vec2; 2],
    pathfinding: &Pathfinding,
) {
    for drone_idx in 0..2 {
        for (i, (_, actions)) in pathfinding.population.iter().enumerate() {
            gc.new_path();

            let mut pos = start_positions[drone_idx];
            gc.move_to(pos.x * 0.1, 1000. - pos.y * 0.1);

            for action in actions {
                pos = pos + action[drone_idx].get_move();
                gc.line_to(pos.x * 0.1, 1000. - pos.y * 0.1);
            }

            gc.line_width(1.0);
            gc.stroke_color(Color::Rgba(
                0.0,
                (drone_idx % 2) as f32,
                ((drone_idx + 1) % 2) as f32,
                if i == 0 { 1. } else { 0.05 },
            ));
            gc.stroke();
        }
    }
}

struct App {
    sim_world: SimWorld,
    world: World,
    strategy: Strategy,
    canvas: Canvas,

    draw_bounds: bool,
    draw_weights: bool,
    draw_paths: bool,
}

impl App {
    fn new(canvas: Canvas) -> Self {
        let sim_world = SimWorld::gen_random_world();
        let world = sim_world.build_world();
        let strategy = Strategy::new();

        App {
            sim_world,
            world,
            strategy,
            canvas: canvas,
            draw_bounds: false,
            draw_weights: false,
            draw_paths: false,
        }
    }

    fn redraw(&mut self) {
        self.canvas.draw(|gc| {
            gc.clear_all_layers();
            gc.canvas_height(1000.0);
            gc.center_region(0.0, 0.0, 1000.0, 1000.0);

            draw_lines(gc);

            draw_world(gc, &self.sim_world);

            if self.draw_bounds {
                for bounds in self.strategy.bounds_detector.bounds.values() {
                    draw_bounds(gc, bounds.top_left, bounds.bot_right);
                }
            }

            if self.draw_weights {
                draw_weights(gc, &self.strategy.score_map, &self.strategy.exploration_map);
            }

            if self.draw_paths {
                draw_paths(
                    gc,
                    [self.sim_world.drones[0].pos, self.sim_world.drones[1].pos],
                    &self.strategy.pathfinding,
                );
            }
        });
    }

    fn regenerate_map(&mut self) {
        self.sim_world = SimWorld::gen_random_world();
        self.world = self.sim_world.build_world();
        self.strategy = Strategy::new();
        self.strategy.play(&self.world);

        self.redraw();
    }
}

fn main() {
    with_2d_graphics(|| {
        executor::block_on(async {
            let (canvas, mut events) = create_canvas_window_with_events("CGBOT");

            let mut app = App::new(canvas);

            app.redraw();

            while let Some(event) = events.next().await {
                match event {
                    DrawEvent::KeyDown(_, Some(Key::KeySpace)) => {
                        app.regenerate_map();
                    }
                    DrawEvent::KeyDown(_, Some(Key::KeyEscape)) => {
                        std::process::exit(0);
                    }
                    DrawEvent::KeyDown(_, Some(Key::Key1)) => {
                        app.draw_bounds = !app.draw_bounds;
                        app.redraw();
                    }
                    DrawEvent::KeyDown(_, Some(Key::Key2)) => {
                        app.draw_weights = !app.draw_weights;
                        app.redraw();
                    }
                    DrawEvent::KeyDown(_, Some(Key::Key3)) => {
                        app.draw_paths = !app.draw_paths;
                        app.redraw();
                    }
                    _ => {}
                }
            }
        });
    });
}
