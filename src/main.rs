use aetherfall::{Material, World};

fn main() {
    let mut world = World::new(24, 12).expect("demo world dimensions are valid");

    for x in 0..world.width() {
        world.set(x, world.height() - 1, Material::Stone);
    }

    for x in 7..17 {
        world.set(x, 8, Material::Wood);
    }

    for x in 4..10 {
        world.set(x, 1, Material::Sand);
    }

    for x in 14..20 {
        world.set(x, 2, Material::Water);
    }

    world.set(12, 7, Material::Fire);

    for _ in 0..8 {
        world.step();
    }

    println!("Aetherfall tick {}", world.tick());
    println!("{}", world.render_ascii());
}
