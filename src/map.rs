use std::{collections::HashMap, ops::Range};

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

use crate::{Args, biome::Biome, utils};

/// An entire game map, effectively a voronoi diagram.
#[derive(Component)]
#[require(Entropy<WyRand>, Transform)]
pub struct Map {
    pub size: Vec2,

    pub sector_num: u32,

    pub lloyd_iters: u32,
    pub generator_border: f32,
    pub altitude_perlin_scale: f32,

    /// A list of all the sectors in the map.
    pub sectors: Vec<Entity>,

    pub biome_seed_num: u32,

    pub city_num: u32,
    pub city_min_spacing: f32,
    pub city_start_pop_range: Range<u32>,
    pub city_deadzone: f32,

    pub nodes_per_city_range: Range<u32>,
    pub node_city_max_dist: f32,
    pub node_city_min_dist: f32,
    pub node_min_spacing: f32,
    pub node_deadzone: f32,
}

/// A single polygon in the voronoi diagram.
#[derive(Debug, Component)]
#[require(Mesh2d, Transform)]
pub struct Sector {
    /// The point that spawned this sector in the voronoi diagram.
    pub site: Vec2,

    /// The points making up the boundary of this sector.
    pub border: Vec<Vec2>,

    /// The centroid of this site's polygon.
    pub centroid: Vec2,

    /// The height of this sector on the map.
    pub height: f32,

    pub biome: Option<Biome>,

    /// The cost per unit of traversing this sector (for road pathfinding).
    pub cost: f32,

    /// This sector's neighbors.
    pub neighbors: Vec<Entity>,
}

pub fn generate_map(
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
    for _ in 0..10000 {
        let new_site = Vec2 {
            x: rng.random_range(0.0..(map.size.x.round() + map.generator_border * 2.0))
                - map.generator_border,
            y: rng.random_range(0.0..(map.size.y.round() + map.generator_border * 2.0))
                - map.generator_border,
        };
        if sites.iter().any(|site| site.distance(new_site) < 1.0) {
            continue;
        }
        sites.push(new_site);
        if sites.len() >= map.sector_num as usize {
            break;
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

    println!("Sites after building mesh: {}", voronoi.sites().len());

    // Maps VoronoiCell site indices to entity ID's
    let mut site_to_entity: HashMap<usize, Entity> = HashMap::new();

    // Maps VoronoiCell site indices to sectors
    let mut site_to_sector: HashMap<usize, Sector> = HashMap::new();

    for (site_index, cell) in voronoi.iter_cells().enumerate() {
        let vertices: Vec<Vec2> = cell.iter_vertices().map(point_to_vec2).collect();

        let site = point_to_vec2(cell.site_position());
        let height = perlin_noise.sample(site * map.altitude_perlin_scale);
        let centroid = utils::centroid(&vertices);
        let sector = Sector {
            site,
            centroid,
            border: vertices,
            height,
            biome: None,
            cost: 1.0,
            neighbors: Vec::new(),
        };

        let sector_entity = commands.spawn(()).id();

        map.sectors.push(sector_entity);
        commands.entity(entity).add_child(sector_entity);

        site_to_entity.insert(site_index, sector_entity);
        site_to_sector.insert(site_index, sector);
    }

    for (site_index, cell) in voronoi.iter_cells().enumerate() {
        let sector = site_to_sector.get_mut(&site_index).unwrap();
        for neighbor_site_index in cell.iter_neighbors() {
            sector
                .neighbors
                .push(*site_to_entity.get(&neighbor_site_index).unwrap());
        }

        commands
            .entity(*site_to_entity.get(&site_index).unwrap())
            .insert(site_to_sector.remove(&site_index).unwrap());
    }
}

pub fn add_map_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    map_query: Query<(Entity, &Map)>,
    sector_query: Query<&Sector>,
) {
    // Data to be added to map mesh
    let mut positions: Vec<Vec3> = Vec::new();
    let mut triangles: Vec<u32> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();

    let (e_map, map) = map_query.single().unwrap();

    for e_sector in &map.sectors {
        let sector = sector_query.get(*e_sector).unwrap();

        let index_offset = positions.len() as u32;
        for i in 1..sector.border.len() - 1 {
            triangles.push(index_offset as u32);
            triangles.push(i as u32 + index_offset);
            triangles.push(i as u32 + index_offset + 1);
        }
        let color = match sector.biome.unwrap() {
            Biome::Plains => Color::srgb_u8(255, 250, 205).darker(sector.height / 2.0),
            Biome::Forest => Color::srgb_u8(34, 139, 34).darker(sector.height / 8.0),
            Biome::Desert => Color::srgb_u8(255, 255, 224).darker(sector.height / 2.0),
        };
        for vertex in &sector.border {
            positions.push(vertex.extend(0.0));
            colors.push(color.to_linear().to_f32_array());
        }
    }

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(triangles));
    let mesh_handle = meshes.add(mesh);
    let mesh_entity = commands
        .spawn((
            Mesh2d(mesh_handle),
            MeshMaterial2d(materials.add(Color::WHITE)),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();
    commands.entity(e_map).add_child(mesh_entity);
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
        // gizmos.circle_2d(sector.site, 3.0, Color::WHITE);
        gizmos.circle_2d(sector.centroid, 3.0, Color::srgb(0.0, 1.0, 1.0));

        for neighbor in &sector.neighbors {
            gizmos.line_2d(
                sector.centroid,
                sector_query.get(*neighbor).unwrap().centroid,
                Color::WHITE,
            );
        }
    }

    let map = map_query.single().unwrap();
    gizmos.rect_2d(map.size / 2.0, map.size, Color::WHITE);
}

pub fn create_map(mut commands: Commands, args: Res<Args>) {
    let seed: u64 = args.seed;

    commands.spawn((
        Map {
            size: vec2(1000.0, 500.0),

            sector_num: 8000,

            lloyd_iters: 5,
            generator_border: 20.0,
            altitude_perlin_scale: 0.008,

            sectors: Vec::new(),

            biome_seed_num: 60,

            city_num: 10,
            city_min_spacing: 100.0,
            city_start_pop_range: 10..1000,
            city_deadzone: 20.0,

            nodes_per_city_range: 1..3,
            node_city_max_dist: 150.0,
            node_city_min_dist: 20.0,
            node_min_spacing: 70.0,
            node_deadzone: 10.0,
        },
        Entropy::<WyRand>::seed_from_u64(seed),
        Transform::IDENTITY,
        Visibility::Visible,
    ));
}
