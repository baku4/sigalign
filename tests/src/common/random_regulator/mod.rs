use rand::Rng;

pub fn gen_random_regulator() -> (u32, u32, u32, u32, f32) {
    let mut rng = rand::thread_rng();

    let px = rng.gen_range(1..10);
    let po = rng.gen_range(0..10);
    let pe = rng.gen_range(1..10);
    
    let minl = rng.gen_range(50..150);
    let maxp = {
        (
            px * rng.gen_range(1..5) // mismatch per 100 bases
        ) as f32 / 100.0
    };
    (px, po, pe, minl, maxp)
}
