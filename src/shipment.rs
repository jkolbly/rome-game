use bevy::prelude::*;

use crate::resource::Resource;

/// Struct carrying data about a shipment.
#[derive(Clone, Copy)]
pub struct Shipment {
    pub resource: Resource,
    pub quantity: u32,
}

/// Component for entities that can receive resource shipments.
#[derive(Component)]
pub struct ShipmentReceiver {
    incoming: Vec<Shipment>,
}

impl Default for ShipmentReceiver {
    fn default() -> Self {
        Self::new()
    }
}

impl ShipmentReceiver {
    pub fn new() -> ShipmentReceiver {
        ShipmentReceiver {
            incoming: Vec::new(),
        }
    }

    /// Return whether there are any unprocessed arrived shipments.
    pub fn has_incoming(&self) -> bool {
        !self.incoming.is_empty()
    }

    /// Remove and return the next arrived shipment if there is one.
    pub fn get_shipment(&mut self) -> Option<Shipment> {
        self.incoming.pop()
    }

    /// Add a shipment to this receiver
    pub fn add_shipment(&mut self, shipment: Shipment) {
        self.incoming.push(shipment);
    }
}
