use godot::prelude::*;

use crate::player::Player;

#[derive(GodotClass)]
#[class(base=Camera3D)]
struct PlayerCamera {
    #[export]
    follow_smoothing: f32,
    #[export]
    target: Option<Gd<Node3D>>,
    #[export]
    player: Option<Gd<Player>>,
    base: Base<Camera3D>,
}

#[godot_api]
impl ICamera3D for PlayerCamera {
    fn init(base: Base<Camera3D>) -> Self {
        Self {
            follow_smoothing: 1.0,
            target: None,
            player: None,
            base,
        }
    }

    fn ready(&mut self) {
        if let Some(player) = self.get_player() {
            self.set_target(player.bind().get_camera_target());
        }
    }

    fn process(&mut self, delta: f64) {
        if let Some(ref target) = self.get_target() {
            /*
            let current_rot = self.base().get_global_transform().basis.to_quat();
            let target_rot = target.get_global_transform().basis.to_quat();
            let interp_rot = current_rot.slerp(target_rot, self.follow_smoothing * delta as f32);
            self.base_mut().set_global_rotation(interp_rot.to_euler(EulerOrder::XYZ));

            let target_position = target.get_global_position();
            let current_pos = self.base().get_global_position();
            let interp_pos =
                current_pos.slerp(target_position, self.follow_smoothing * delta as f32);
            self.base_mut().set_global_position(interp_pos);
             */
            let current_basis = self.base().get_global_basis();
            let target_basis = target.get_global_basis();
            let inter_basis =
                current_basis.slerp(&target_basis, delta as f32 * self.follow_smoothing);
            self.base_mut().set_global_basis(inter_basis);

            let target_position = target.get_global_position();
            let current_pos = self.base().get_global_position();
            let interp_pos =
                current_pos.slerp(target_position, self.follow_smoothing * delta as f32);
            self.base_mut().set_global_position(interp_pos);
        }
    }
}

#[godot_api]
impl PlayerCamera {}
