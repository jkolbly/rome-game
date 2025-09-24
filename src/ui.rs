use bevy::prelude::*;


/// Component for UI displayed relative to the world.
#[derive(Component, Clone)]
#[require(Node)]
pub struct UIWorldPosition {
    pub pos: Vec2,
}

pub fn update_world_ui_positions(
    q_camera: Query<(Entity, &Camera)>,
    q_node: Query<(&UIWorldPosition, &mut Node)>,
    transform_helper: TransformHelper,
) {
    let (e_camera, camera) = q_camera.single().unwrap();
    let global_t_camera = transform_helper.compute_global_transform(e_camera).unwrap();

    for (pos, mut node) in q_node {
        let screen_pos = camera
            .world_to_viewport(&global_t_camera, pos.pos.extend(0.0))
            .unwrap();
        node.left = Val::Px(screen_pos.x);
        node.top = Val::Px(screen_pos.y);
    }
}
