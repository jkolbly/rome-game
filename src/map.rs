use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use bevy_prng::WyRand;
use bevy_rand::prelude::Entropy;
use noiz::prelude::*;
use rand::{Rng, SeedableRng};
use voronoice::*;

use crate::{
    biome::Biome,
    clickable::{ClickHitbox, ClickState, JustPressed},
    utils,
};

/// An entire game map, effectively a voronoi diagram.
#[derive(Component)]
#[require(Entropy<WyRand>, Transform)]
pub struct Map {
    size: Vec2,

    sector_num: u32,

    lloyd_iters: u32,
    generator_border: f32,
    height_perlin_scale: f32,

    /// A list of all the sectors in the map.
    sectors: Vec<Entity>,
}

/// A single polygon in the voronoi diagram.
#[derive(Debug, Component)]
#[require(Mesh2d, Transform)]
pub struct Sector {
    /// The point that spawned this sector in the voronoi diagram.
    site: Vec2,

    /// The centroid of this site's polygon.
    centroid: Vec2,

    /// The height of this sector on the map.
    height: f32,

    biome: Biome,
}

pub fn generate_map(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Map, &mut Entropy<WyRand>)>,
) {
    let (entity, mut map, mut rng) = query.single_mut().unwrap();

    commands.entity(entity).despawn_related::<Children>();

    let mut perlin_noise = Noise::<(
        MixCellGradients<OrthoGrid, Smoothstep, QuickGradients>,
        SNormToUNorm,
    )>::default();
    perlin_noise.set_seed(rng.random());

    println!("Generating map with size: {}", map.size);

    let mut sites: Vec<Vec2> = Vec::new();
    for _ in 0..map.sector_num {
        let site = Vec2 {
            x: rng.random_range(..(map.size.x.round() + map.generator_border * 2.0) as u32) as f32
                - map.generator_border,
            y: rng.random_range(..(map.size.y.round() + map.generator_border * 2.0) as u32) as f32
                - map.generator_border,
        };
        if !sites.contains(&site) {
            sites.push(site);
        }
    }
    println!("Generated {} sites", sites.len());

    let boundary = BoundingBox::new(
        vec2_to_point(&(map.size / 2.0)),
        map.size.x as f64,
        map.size.y as f64,
    );
    let voronoi = VoronoiBuilder::default()
        .set_sites(sites.iter().map(vec2_to_point).collect())
        .set_bounding_box(boundary)
        .set_clip_behavior(ClipBehavior::Clip)
        .set_lloyd_relaxation_iterations(map.lloyd_iters as usize)
        .build()
        .unwrap();

    for cell in voronoi.iter_cells() {
        let vertices: Vec<Vec2> = cell.iter_vertices().map(point_to_vec2).collect();

        let site = point_to_vec2(cell.site_position());
        let height = perlin_noise.sample(site * map.height_perlin_scale);
        let centroid = utils::centroid(&vertices);
        let sector = Sector {
            site,
            centroid,
            height,
            biome: Biome::Plains,
        };

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertices
                .iter()
                .map(|v| v.extend(0.0))
                .collect::<Vec<Vec3>>(),
        )
        .with_inserted_indices(Indices::U32(
            (1..vertices.len() - 1)
                .map(|i| vec![0u32, i as u32, i as u32 + 1])
                .flatten()
                .collect(),
        ));

        let mesh_handle = meshes.add(mesh);
        let color = Color::srgb(0.5, height, 0.5);
        let material_handle = materials.add(color);
        let sector_entity = commands
            .spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(material_handle),
                Transform::from_xyz(0.0, 0.0, 0.0),
                sector,
            ))
            .id();

        map.sectors.push(sector_entity);
        commands.entity(entity).add_child(sector_entity);
    }
}

fn vec2_to_point(v: &Vec2) -> Point {
    Point {
        x: v.x as f64,
        y: v.y as f64,
    }
}

fn point_to_vec2(p: &Point) -> Vec2 {
    Vec2 {
        x: p.x as f32,
        y: p.y as f32,
    }
}

pub fn draw_debug(mut gizmos: Gizmos, map_query: Query<&Map>, sector_query: Query<&Sector>) {
    for sector in &sector_query {
        gizmos.circle_2d(sector.site, 3.0, Color::WHITE);
        gizmos.circle_2d(sector.centroid, 3.0, Color::srgb(0.0, 1.0, 1.0));
    }

    for map in map_query {
        gizmos.rect_2d(map.size / 2.0, map.size, Color::WHITE);
    }
}

pub fn create_map(mut commands: Commands) {
    let seed: u64 = 1;

    commands.spawn((
        Map {
            size: vec2(500.0, 500.0),

            sector_num: 5000,

            lloyd_iters: 1,
            generator_border: 10.0,
            height_perlin_scale: 0.015,

            sectors: Vec::new(),
        },
        Entropy::<WyRand>::seed_from_u64(seed),
        Transform::IDENTITY,
    ));
}
