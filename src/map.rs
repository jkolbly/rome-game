use bevy::{prelude::*, render::mesh::RectangleMeshBuilder};
use bevy_prng::WyRand;
use bevy_rand::prelude::Entropy;
use rand::{Rng, SeedableRng};

use crate::voronoi;

/// An entire game map, effectively a voronoi diagram.
#[derive(Component)]
pub struct Map {
    width: u32,
    height: u32,

    sector_num: u32,
    sectors: Vec<Sector>,
}

#[derive(Bundle)]
pub struct MapBundle {
    map: Map,
    rng: Entropy<WyRand>,
}

/// A single polygon in the voronoi diagram.
#[derive(Debug)]
pub struct Sector {
    center_x: f32,
    center_y: f32,
}

pub fn generate_map(mut query: Query<(&mut Map, &mut Entropy<WyRand>)>) {
    for (mut map, mut rng) in &mut query {
        println!(
            "Generating map with width: {} height: {}",
            map.width, map.height
        );

        let mut points: Vec<(f32, f32)> = Vec::new();
        for _ in 0..map.sector_num {
            let point = (
                rng.random_range(..map.width) as f32,
                rng.random_range(..map.height) as f32,
            );
            points.push(point);
            println!("Generated point: {:?}", point);
        }

        let sectors = voronoi::voronoi(points);
        println!("Generated sectors: {:?}", sectors);
    }
}

pub fn gen_map_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(Entity, &Map), Without<Mesh2d>>,
) {
    for (entity, map) in &mut query {
        let mesh = Rectangle::new(map.width as f32, map.height as f32);

        let mesh_handle = meshes.add(mesh);
        commands.entity(entity).insert((
            Mesh2d(mesh_handle),
            MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
            Transform::from_xyz(-(map.width as f32) / 2.0, -(map.height as f32) / 2.0, 0.0),
        ));
    }
}

pub fn create_map(mut commands: Commands) {
    let seed: u64 = 1;

    commands.spawn(MapBundle {
        map: Map {
            width: 100,
            height: 100,

            sector_num: 100,
            sectors: Vec::new(),
        },
        rng: Entropy::<WyRand>::seed_from_u64(seed),
    });
}
