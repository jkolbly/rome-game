use std::ops::Range;

use bevy::prelude::*;

#[derive(Resource)]
pub struct MapGenSettings {
    pub size: Vec2,

    pub sector_num: u32,

    pub lloyd_iters: u32,
    pub generator_border: f32,
    pub altitude_perlin_scale: f32,

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

#[derive(Resource)]
pub struct DisplaySettings {
    pub road_width: f32,
}
