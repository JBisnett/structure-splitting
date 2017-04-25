const N_BODIES: usize = 10000000;
// const N_BODIES: usize = 10000;
extern crate rand;
use rand::Rng;
use std::time::{Duration, Instant};

// #[affinity_groups(x=1, y=1, z=1, vx=2, vy=2, vz=2, mass=2)]
struct Planet {
	x: f64,
	y: f64,
	z: f64,
	vx: f64,
	vy: f64,
	vz: f64,
	mass: [f64; 32],
}

fn contrived() {
	let mut rng = rand::thread_rng();
	let mut planets = (0..N_BODIES)
		.map(|_| {
			Planet {
				x: rng.gen(),
				y: rng.gen(),
				z: rng.gen(),
				vx: rng.gen(),
				vy: rng.gen(),
				vz: rng.gen(),
				mass: rng.gen(),
			}
		})
		.collect::<Vec<_>>();
	let timer = Instant::now();
	for i in 0..1000 {
		for j in 0..N_BODIES {
			if i != j {
				planets[i].x = planets[i].x - planets[j].x;
				planets[i].y = planets[i].y - planets[j].y;
				planets[i].z = planets[i].z - planets[j].z;
			}
		}
	}
	let elapsed = timer.elapsed();
	println!{"Time: {}s\t{}ns", elapsed.as_secs(), elapsed.subsec_nanos()};
}

fn main() {
	contrived();
}
