use bevy::{prelude::*, render::mesh::RectangleMeshBuilder};
use bevy_prng::WyRand;
use bevy_rand::prelude::Entropy;
use rand::{Rng, SeedableRng};
use voronoi_mosaic::prelude::*;

use crate::utils;

/// An entire game map, effectively a voronoi diagram.
#[derive(Component)]
pub struct Map {
    size: Vec2,

    sector_num: u32,
    sectors: Vec<Sector>,

    lloyd_iters: u32,
    generator_border: f32,
}

#[derive(Bundle)]
pub struct MapBundle {
    map: Map,
    rng: Entropy<WyRand>,
}

/// A single polygon in the voronoi diagram.
#[derive(Debug)]
pub struct Sector {
    /// The point that spawned this sector in the voronoi diagram.
    site: Vec2,

    /// The centroid of this site's polygon.
    centroid: Vec2,
}

pub fn generate_map(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut query: Query<(&mut Map, &mut Entropy<WyRand>)>,
) {
    for (mut map, mut rng) in &mut query {
        println!("Generating map with size: {}", map.size);

        let mut points: Vec<Vec2> = Vec::new();
        for _ in 0..map.sector_num {
            let point = Vec2 {
                x: rng.random_range(..(map.size.x.round() + map.generator_border * 2.0) as u32)
                    as f32
                    - map.generator_border,
                y: rng.random_range(..(map.size.y.round() + map.generator_border * 2.0) as u32)
                    as f32
                    - map.generator_border,
            };
            if !points.contains(&point) {
                points.push(point);
            }
        }
        println!("Generated {} points", points.len());

        let mut delaunay = DelaunayData::compute_triangulation_2d(&points).unwrap();
        let mut voronoi = VoronoiData::from_delaunay_2d(&delaunay).unwrap();

        for _ in 0..map.lloyd_iters {
            let mut new_points = Vec::new();
            for (_, cell) in voronoi.get_cells() {
                let vertices = cell.get_vertices();
                let centroid = utils::centroid(vertices);
                new_points.push(centroid);
            }

            delaunay = DelaunayData::compute_triangulation_2d(&new_points).unwrap();
            voronoi = VoronoiData::from_delaunay_2d(&delaunay).unwrap();
        }

        let boundary = vec![
            vec2(0.0, 0.0),
            vec2(map.size.x, 0.0),
            map.size,
            vec2(0.0, map.size.y),
        ];
        voronoi.clip_cells_to_boundary(&boundary);
        let vmeshes = voronoi.as_bevy_meshes_2d();

        println!("Created {} meshes", vmeshes.len());

        for (mesh, vec) in vmeshes {
            let mesh_handle = meshes.add(mesh);
            let color = Color::srgb(rng.random(), rng.random(), rng.random());
            let material_handle = materials.add(color);
            commands.spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(material_handle),
                Transform::from_xyz(vec.x, vec.y, 0.0),
            ));
        }

        for (_, cell) in voronoi.get_cells() {
            map.sectors.push(Sector {
                site: *cell.get_generating_point(),
                centroid: utils::centroid(cell.get_vertices()),
            });
        }

        // println!("Generated sectors: {:?}", sectors);
    }
}

pub fn draw_debug(mut gizmos: Gizmos, query: Query<&mut Map>) {
    for map in query {
        for sector in &map.sectors {
            gizmos.circle_2d(sector.site, 3.0, Color::WHITE);
            gizmos.circle_2d(sector.centroid, 3.0, Color::srgb(0.0, 1.0, 1.0));
        }
        gizmos.rect_2d(map.size / 2.0, map.size, Color::WHITE);
    }
}

pub fn create_map(mut commands: Commands) {
    let seed: u64 = 1;

    commands.spawn(MapBundle {
        map: Map {
            size: vec2(500.0, 500.0),

            sector_num: 2000,
            sectors: Vec::new(),

            lloyd_iters: 3,
            generator_border: 50.0,
        },
        rng: Entropy::<WyRand>::seed_from_u64(seed),
    });
}
