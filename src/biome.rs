use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::Entropy;
use rand::Rng;

use crate::map::{Map, Sector};

#[derive(Debug, Clone, Copy)]
pub enum Biome {
    Plains,
    Forest,
    Desert,
}

pub fn generate_biomes(
    mut map_query: Query<(&Map, &mut Entropy<WyRand>)>,
    mut sector_query: Query<&mut Sector>,
) {
    let (map, mut rng) = map_query.single_mut().unwrap();

    let mut seeds: VecDeque<BiomeSeed> = VecDeque::new();

    // Generate random biome seeds
    for _ in 0..1000 {
        let rand_index = rng.random_range(0..map.sectors.len());
        let rand_sector = map.sectors[rand_index];

        // Check this isn't already a seed.
        if seeds.iter().any(|seed| seed.e_sector == rand_sector) {
            continue;
        }

        let biome_type: Biome = match rng.random_range(0..3) {
            0 => Biome::Plains,
            1 => Biome::Forest,
            2 => Biome::Desert,
            _ => unreachable!(),
        };

        seeds.push_back(BiomeSeed {
            biome: biome_type,
            e_sector: rand_sector,
        });

        if seeds.len() >= map.biome_seed_num as usize {
            break;
        }
    }

    // Spread biome seeds breadth-first until there are no more
    while let Some(BiomeSeed { biome, e_sector }) = seeds.pop_front() {
        let mut sector = sector_query.get_mut(e_sector).unwrap();

        // Skip sectors that already have a biome
        if sector.biome.is_some() {
            continue;
        }

        sector.biome = Some(biome);

        for neighbor in &sector.neighbors {
            seeds.push_back(BiomeSeed {
                biome,
                e_sector: *neighbor,
            });
        }
    }
}

/// Helper struct used during biome generation.
struct BiomeSeed {
    biome: Biome,
    e_sector: Entity,
}
