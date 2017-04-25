const N_BODIES: usize = 1000000;
// const N_BODIES: usize = 10000;
extern crate rand;
use rand::Rng;
use std::time::{Duration, Instant};

// #[affinity_groups(x=1, y=1, z=1, vx=2, vy=2, vz=2, mass=2)]
struct Planet1 {
	x: f64,
	y: f64,
	z: f64,
}
struct Planet2 {
	vx: f64,
	vy: f64,
	vz: f64,
	mass: [f64; 32],
}

fn contrived() {
	let mut rng = rand::thread_rng();
	let mut planets: (Vec<_>, Vec<_>) = (0..N_BODIES)
		.map(|_| {
			(Planet1 {
				x: rng.gen(),
				y: rng.gen(),
				z: rng.gen(),
			},
			 Planet2 {
				vx: rng.gen(),
				vy: rng.gen(),
				vz: rng.gen(),
				mass: rng.gen(),
			})
		})
		.unzip();
	let timer = Instant::now();
	for i in 0..1000 {
		for j in 0..N_BODIES {
			if i != j {
				planets.0[i].x = planets.0[i].x - planets.0[j].x;
				planets.0[i].y = planets.0[i].y - planets.0[j].y;
				planets.0[i].z = planets.0[i].z - planets.0[j].z;
			}
		}
	}
	let elapsed = timer.elapsed();
	println!{"Time: {}s\t{}ns", elapsed.as_secs(), elapsed.subsec_nanos()};
}

fn main() {
	contrived();
}
