use bevy::prelude::*;

use crate::{
    click_off::Despawning,
    resource::Resource,
    road::Road,
    settings::GameplaySettings,
    shipment::{Shipment, ShipmentReceiver},
};

/// Component for wagons that carry [`Shipment`]s.
#[derive(Component)]
#[require(Transform)]
pub struct Wagon {
    shipment: Shipment,
    road: Entity,
    distance: f32,
    destination: Entity,
}

impl Wagon {
    pub fn new(shipment: Shipment, road: Entity, destination: Entity) -> Wagon {
        Wagon {
            shipment,
            road,
            distance: 0.0,
            destination,
        }
    }
}

pub fn move_wagons(
    time: Res<Time>,
    settings: Res<GameplaySettings>,
    mut commands: Commands,
    wagon_query: Query<(Entity, &mut Wagon, &mut Transform)>,
    road_query: Query<&Road>,
    mut dest_query: Query<&mut ShipmentReceiver>,
) {
    for (e_wagon, mut wagon, mut t_wagon) in wagon_query {
        let road = road_query.get(wagon.road).unwrap();
        wagon.distance += time.delta_secs() * road.speed_multiplier * settings.wagon_speed;
        if wagon.distance < road.length {
            // Wagon is still travelling

            // Make sure we sample from the end if we're moving backwards
            let sample_dist = if road.end_sector == wagon.destination {
                wagon.distance
            } else {
                road.length - wagon.distance
            };

            let new_pos = road.curve.sample(sample_dist).unwrap();
            t_wagon.translation = new_pos.extend(0.0);
        } else {
            // Wagon has arrived
            let mut receiver = dest_query.get_mut(wagon.destination).unwrap();
            receiver.add_shipment(wagon.shipment);

            commands.entity(e_wagon).insert(Despawning);
        }
    }
}

pub fn add_wagon_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    wagon_query: Query<(Entity, &Wagon), Without<Mesh2d>>,
) {
    for (e_wagon, wagon) in wagon_query {
        let color = match wagon.shipment.resource {
            Resource::Wheat => Color::srgb_u8(218, 165, 32),
            Resource::Ore => Color::srgb_u8(112, 128, 144),
            Resource::Lumber => Color::srgb_u8(139, 69, 19),
        };

        let material = materials.add(color);
        let mesh = meshes.add(Circle::new(2.5));

        commands
            .entity(e_wagon)
            .insert((Mesh2d(mesh), MeshMaterial2d(material)));
    }
}
