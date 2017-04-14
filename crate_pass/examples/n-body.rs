#![feature(test, plugin, custom_derive)]
#![plugin(compiler)]
const N_BODIES: usize = 100000;
extern crate rand;
use rand::Rng;
//#[affinity_groups(x=1, y=1, z=1, vx=2, vy=2, vz=2, mass=2)]
#[derive(Clone)]
struct Planet {
  x: f64,
  y: f64,
  z: f64,
  vx: f64,
  vy: f64,
  vz: f64,
  mass: f64,
}

impl Copy for Planet {}

fn advance(dt: f64, steps: i32) {
  let mut bodies = [Planet {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    vx: 0.0,
    vy: 0.0,
    vz: 0.0,
    mass: 0.0,
  }; N_BODIES];

  let mut rng = rand::thread_rng();

  for i in 0..N_BODIES {
    bodies[i].x = rng.gen();
    bodies[i].y = rng.gen();
    bodies[i].z = rng.gen();
    bodies[i].vx = rng.gen();
    bodies[i].vy = rng.gen();
    bodies[i].vz = rng.gen();
    bodies[i].mass = rng.gen();
  }
  for _ in 0..steps {
    for i in 0..N_BODIES {
      let mut dxs = [0.0; N_BODIES];
      let mut dys = [0.0; N_BODIES];
      let mut dzs = [0.0; N_BODIES];
      for j in 0..N_BODIES {
        if i != j {
          dxs[i] = bodies[i].x - bodies[j].x;
          dys[i] = bodies[i].y - bodies[j].y;
          dzs[i] = bodies[i].z - bodies[j].z;
        }
      }
      for j in 0..N_BODIES {
        let dx = dxs[i];
        let dy = dys[i];
        let dz = dzs[i];

        let d2 = dx * dx + dy * dy + dz * dz;
        let mag = dt / (d2 * d2.sqrt());

        let massj_mag = bodies[j].mass * mag;
        bodies[i].vx -= dx * massj_mag;
        bodies[i].vy -= dy * massj_mag;
        bodies[i].vz -= dz * massj_mag;

        let massi_mag = bodies[i].mass * mag;
        bodies[j].vx += dx * massi_mag;
        bodies[j].vy += dy * massi_mag;
        bodies[j].vz += dz * massi_mag;
      }
      bodies[i].x += dt * bodies[i].vx;
      bodies[i].y += dt * bodies[i].vy;
      bodies[i].z += dt * bodies[i].vz;
    }
  }
}

fn main() {
  advance(0.01, 1);
}
