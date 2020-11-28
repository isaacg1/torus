use image::{ImageBuffer, Rgb, RgbImage};
use rand::prelude::*;

// Geometry is a square torus
// Colors have neutral point at 128
// Attract or repel.

type Point = (f64, f64);
type Vector = (f64, f64);
type Color = (f64, f64, f64);
const NEUTRAL: f64 = 128.0;

type Particle = (Point, Color);

fn inv_r2(x: Point, y: Point, size: f64) -> Vector {
    let hdir = (x.0 - y.0).signum();
    let vdir = (x.1 - y.1).signum();
    let ypoints = vec![
        y,
        (y.0 + hdir * size, y.1),
        (y.0, y.1 + vdir * size),
        (y.0 + hdir * size, y.1 + vdir * size),
    ];
    let mut net_dir = (0.0, 0.0);
    for yi in ypoints {
        let vec = (yi.0 - x.0, yi.1 - x.1);
        let vec_len = vec.0.hypot(vec.1);
        // 1 vec_len to make a unit vector,
        // 2 vec_len to make gravity.
        let force = (vec.0 / vec_len.powi(3), vec.1 / vec_len.powi(3));
        net_dir = (net_dir.0 + force.0, net_dir.1 + force.1);
    }
    net_dir
}

fn color_force(a: Color, b: Color) -> f64 {
    let f0 = (a.0 - NEUTRAL) * (b.0 - NEUTRAL);
    let f1 = (a.1 - NEUTRAL) * (b.1 - NEUTRAL);
    let f2 = (a.2 - NEUTRAL) * (b.2 - NEUTRAL);
    f0 + f1 + f2
}

fn norm(unnorm: Point, size: f64) -> Point {
    (modulus(unnorm.0, size), modulus(unnorm.1, size))
}

// Map a to range [0, b)
fn modulus(a: f64, b: f64) -> f64 {
    a - (a / b).floor() * b
}

fn movements(a: Particle, b: Particle, size: f64, g_const: f64) -> (Vector, Vector) {
    let pos_force: Vector = inv_r2(a.0, b.0, size);
    let clr_force: f64 = color_force(a.1, b.1);
    let a_movement: Vector = (
        pos_force.0 * clr_force * g_const,
        pos_force.1 * clr_force * g_const,
    );
    let b_movement: Vector = (-a_movement.0, -a_movement.1);
    (a_movement, b_movement)
}

fn update_color(a: f64, b: f64, weight: f64) -> f64 {
    a + (b - NEUTRAL) * weight
}

fn draw(
    num_particles: usize,
    num_perms: usize,
    num_steps: usize,
    size: usize,
    g_const: f64,
    weight: f64,
    seed: u64,
) -> RgbImage {
    let mut img = vec![vec![[NEUTRAL as f64; 3]; size]; size];
    let mut rng = StdRng::seed_from_u64(seed);
    let fsize = size as f64;
    let mut particles: Vec<Particle> = (0..num_particles)
        .map(|_| {
            (
                (rng.gen_range(0.0, fsize), rng.gen_range(0.0, fsize)),
                (
                    rng.gen_range(0.0, 255.0),
                    rng.gen_range(0.0, 255.0),
                    rng.gen_range(0.0, 255.0),
                ),
            )
        })
        .collect();
    for _ in 0..num_steps {
        let mut net_update_vectors = vec![(0.0, 0.0); num_particles];
        for _ in 0..num_perms {
            let mut indices: Vec<usize> = (0..num_particles).collect();
            indices.shuffle(&mut rng);
            for (i, j) in (0..num_particles).zip(indices) {
                if i == j {
                    continue;
                } else {
                    let updates = movements(particles[i], particles[j], fsize, g_const);
                    net_update_vectors[i] = (
                        net_update_vectors[i].0 + updates.0 .0,
                        net_update_vectors[i].1 + updates.0 .1,
                    );
                    net_update_vectors[j] = (
                        net_update_vectors[j].0 + updates.1 .0,
                        net_update_vectors[j].1 + updates.1 .1,
                    );
                }
            }
        }
        // Update
        for (p, net) in particles.iter_mut().zip(net_update_vectors) {
            let new_pos = (p.0 .0 + net.0, p.0 .1 + net.1);
            let norm_new_pos = norm(new_pos, fsize);
            *p = (norm_new_pos, p.1);
        }
        // Draw
        for p in &particles {
            let pos = (p.0 .0.floor() as usize, p.0 .1.floor() as usize);
            let old_color = img[pos.0][pos.1];
            let [cr, cg, cb] = old_color;
            let new_r = update_color(cr, p.1 .0, weight);
            let new_g = update_color(cg, p.1 .1, weight);
            let new_b = update_color(cb, p.1 .2, weight);
            img[pos.0][pos.1] = [new_r, new_g, new_b];
        }
    }
    ImageBuffer::from_fn(size as u32, size as u32, |i, j| {
        let color = img[i as usize][j as usize];
        Rgb([
            color[0].min(255.0).max(0.0).floor() as u8,
            color[1].min(255.0).max(0.0).floor() as u8,
            color[2].min(255.0).max(0.0).floor() as u8,
        ])
    })
}

fn main() {
    let seed = 0;
    let img = draw(30, 10, 3000000, 1024, 3e-2, 3e-2, seed);
    img.save(format!("img-{}.png", seed))
        .expect("Save succeeds");
}
