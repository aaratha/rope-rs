use macroquad::prelude::*;
use rand::gen_range;
use std::time::{Duration, Instant};

const NUM_PARTICLES: usize = 10;
const SEGMENT_LENGTH: f32 = 10.0;
const TIME_STEP: f32 = 0.016;
const CONSTRAINT_ITERATIONS: usize = 8;
const ENEMY_SPEED: f32 = 10.0;
const FRICTION: f32 = 0.98;
const ENEMY_SPAWN_INTERVAL: Duration = Duration::from_secs(2);
const SPAWN_INTERVAL: Duration = Duration::from_secs(1);
const MAX_POINTS: usize = 20;
const SUBSTEPS: usize = 5;

#[derive(Clone, Copy)]
struct Particle {
    position: Vec2,
    old_position: Vec2,
    acceleration: Vec2,
    friction: f32,
}

impl Particle {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            old_position: position,
            acceleration: Vec2::ZERO,
            friction: FRICTION,
        }
    }

    fn update(&mut self) {
        let mut velocity = self.position - self.old_position;
        velocity *= self.friction; // Apply friction to the velocity
        self.old_position = self.position;
        self.position += velocity; // + self.acceleration * TIME_STEP * TIME_STEP;
        self.acceleration = Vec2::ZERO; // Reset acceleration
    }
}

struct Rope {
    particles: Vec<Particle>,
}

impl Rope {
    fn new(start: Vec2) -> Self {
        let mut particles = Vec::with_capacity(NUM_PARTICLES);
        for i in 0..NUM_PARTICLES {
            particles.push(Particle::new(start + vec2(i as f32 * SEGMENT_LENGTH, 0.0)));
        }
        Self { particles }
    }

    fn update(&mut self, target: Vec2) {
        self.particles[0].position = target;

        for _ in 0..CONSTRAINT_ITERATIONS {
            for i in 0..NUM_PARTICLES - 1 {
                let particle_a = self.particles[i];
                let particle_b = self.particles[i + 1];
                let delta = particle_b.position - particle_a.position;
                let delta_length = delta.length();
                let diff = (delta_length - SEGMENT_LENGTH) / delta_length;
                let offset = delta * diff * 0.5 / SUBSTEPS as f32;

                if i != 0 {
                    self.particles[i].position += offset;
                }
                self.particles[i + 1].position -= offset;
            }
        }

        for i in 1..NUM_PARTICLES {
            self.particles[i].update();
        }
    }

    fn draw(&self) {
        for i in 0..NUM_PARTICLES - 1 {
            draw_line(
                self.particles[i].position.x,
                self.particles[i].position.y,
                self.particles[i + 1].position.x,
                self.particles[i + 1].position.y,
                2.0,
                WHITE,
            );
        }
        draw_circle(
            self.particles[0].position.x,
            self.particles[0].position.y,
            7.0,
            WHITE,
        );
        draw_circle(
            self.particles[NUM_PARTICLES - 1].position.x,
            self.particles[NUM_PARTICLES - 1].position.y,
            7.0,
            WHITE,
        );
    }
}

struct Enemy {
    particle: Particle,
    active: bool,
}

impl Enemy {
    fn new(position: Vec2) -> Self {
        Self {
            particle: Particle::new(position),
            active: true,
        }
    }

    fn update(&mut self, target: Vec2) {
        let direction = target - self.particle.position;
        let distance = direction.length();
        if distance > 0.0 {
            let step = direction.normalize() * ENEMY_SPEED * TIME_STEP;
            self.particle.position += step;
        }
        self.particle.update();
    }

    fn draw(&self) {
        if self.active {
            draw_circle(
                self.particle.position.x,
                self.particle.position.y,
                10.0,
                Color::new(0.7, 0.8, 1.0, 1.0),
            );
        }
    }
}

struct Point {
    position: Vec2,
    active: bool,
    radius: f32,
}

impl Point {
    fn new() -> Self {
        let pos = Vec2::new(
            gen_range(0., screen_width()),
            gen_range(0., screen_height()),
        );
        Self {
            position: pos,
            active: true,
            radius: 5.0,
        }
    }

    fn draw(&self) {
        if self.active {
            draw_circle(
                self.position.x,
                self.position.y,
                self.radius,
                Color::new(1.0, 0.8, 5.0, 5.0),
            );
        }
    }
}

fn check_collisions(
    rope: &mut Rope,
    enemies: &mut [Enemy],
    points: &mut Vec<Point>,
    score: &mut i32,
) {
    for _ in 0..5 {
        for particle in rope.particles.iter_mut() {
            for enemy in enemies.iter_mut() {
                let dist = enemy.particle.position - particle.position;
                let len = dist.length();
                if len < 7.0 + 10.0 {
                    let offset = (7.0 + 10.0 - len) * dist.normalize();
                    enemy.particle.position += offset * 0.5;
                    particle.position -= offset * 0.5;
                }
            }
            for point in points.iter_mut() {
                let dist = point.position - particle.position;
                let len = dist.length();
                if len < 5.0 + 10.0 {
                    point.active = false;
                    *score += 1;
                }
            }
        }
    }
    points.retain(|point| point.active);
}

fn check_enemy_collisions(enemies: &mut [Enemy]) {
    for i in 0..enemies.len() {
        for j in (i + 1)..enemies.len() {
            let dist = enemies[j].particle.position - enemies[i].particle.position;
            let len = dist.length();
            if len < 10.0 * 2.0 {
                let offset = (10.0 * 2.0 - len) * dist.normalize();
                enemies[i].particle.position -= offset * 0.5;
                enemies[j].particle.position += offset * 0.5;
            }
        }
    }
}

fn draw_ring(rope: &Rope) {
    let center = rope.particles[0].position;
    let radius = 200.0; // Adjust the radius as needed
    let color = Color::new(1.0, 1.0, 1.0, 0.5); // Adjust the color and alpha as needed
    draw_circle_lines(center.x, center.y, radius, 2.0, color); // Adjust the line thickness as needed
}

#[macroquad::main("Rope Simulation")]
async fn main() {
    let mut rope = Rope::new(vec2(0.0, 100.0));
    let mut enemies: Vec<Enemy> = Vec::new();
    let mut points: Vec<Point> = Vec::new();
    let mut last_spawn_time = Instant::now();
    let mut last_point_spawn_time = Instant::now();
    let mut score = 0;

    loop {
        let mouse_position: Vec2 = mouse_position().into();
        let target =
            rope.particles[0].position + (mouse_position - rope.particles[0].position) * 0.4;

        for _ in 0..SUBSTEPS {
            rope.update(target);
            check_collisions(&mut rope, &mut enemies, &mut points, &mut score);
            check_enemy_collisions(&mut enemies);
            for enemy in &enemies {
                enemy.draw();
            }
            for point in &points {
                point.draw();
            }
        }

        if last_spawn_time.elapsed() >= ENEMY_SPAWN_INTERVAL {
            enemies.push(Enemy::new(vec2(100.0, 100.0)));
            last_spawn_time = Instant::now();
        }

        if last_point_spawn_time.elapsed() >= SPAWN_INTERVAL && points.len() < MAX_POINTS {
            points.push(Point::new());
            last_point_spawn_time = Instant::now();
        }

        for enemy in &mut enemies {
            enemy.update(rope.particles[0].position);
        }

        for enemy in &mut enemies {
            enemy.particle.update();
        }

        rope.draw();

        draw_text(&format!("Score: {}", score), 20.0, 20.0, 30.0, WHITE);

        draw_ring(&rope);
        next_frame().await;
    }
}
