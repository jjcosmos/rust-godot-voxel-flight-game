use godot::classes::{IRigidBody3D, RigidBody3D, Timer};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=RigidBody3D)]
pub struct Player {
    #[export]
    roll_speed_h: f32,
    #[export]
    pitch_speed_v: f32,
    #[export]
    yaw_speed_h: f32,
    base: Base<RigidBody3D>,
    #[export]
    pub camera_target: Option<Gd<Node3D>>,
    #[export]
    forward_force: f32,
    #[export]
    explosion_scene: Option<Gd<PackedScene>>,
    #[export]
    respawn_timer: Option<Gd<Timer>>,

    #[export]
    close_points_per_sec: f32,
    #[export]
    far_points_per_sec: f32,

    score: f32,
    close_overlap: Option<Gd<Node>>,
    far_overlap: Option<Gd<Node>>,

    #[export]
    pub is_dead: bool,
}

#[godot_api]
impl IRigidBody3D for Player {
    fn init(base: Base<RigidBody3D>) -> Self {
        Self {
            roll_speed_h: 100.0,
            pitch_speed_v: 100.0,
            yaw_speed_h: 100.0,
            camera_target: None,
            forward_force: 5000.0,
            explosion_scene: None,
            respawn_timer: None,
            close_overlap: None,
            far_overlap: None,
            close_points_per_sec: 10.0,
            far_points_per_sec: 5.0,
            score: 0.0,
            is_dead: false,
            base,
        }
    }

    fn ready(&mut self) {
        let callable = Callable::from_object_method(&self.to_gd(), "on_collision");
        self.base_mut().connect("body_entered".into(), callable);

        if let Some(mut timer) = self.get_respawn_timer() {
            let timer_callback = Callable::from_object_method(&self.to_gd(), "on_timer_timeout");
            timer.connect("timeout".into(), timer_callback);
        }

        self.base_mut().set_freeze_enabled(true);
    }

    fn process(&mut self, delta: f64) {
        let input = Input::singleton();
        // Inverted Y by default
        let vertical_input = input.get_axis("v_negative".into(), "v_positive".into());
        let horizontal_input: f32 = input.get_axis("h_negative".into(), "h_positive".into());
        let horizontal2_input: f32 = input.get_axis("h2_negative".into(), "h2_positive".into());

        let torque_roll = self.base().get_global_basis().col_c()
            * horizontal2_input
            * delta as f32
            * self.get_roll_speed_h();
        self.base_mut().apply_torque(-torque_roll);

        let torque_yaw = self.base().get_global_basis().col_b()
            * horizontal_input
            * delta as f32
            * self.get_yaw_speed_h();
        self.base_mut().apply_torque(-torque_yaw);

        let torque_pitch = self.base().get_global_basis().col_a()
            * vertical_input
            * delta as f32
            * self.get_pitch_speed_v();
        self.base_mut().apply_torque(-torque_pitch);

        if !self.base().is_freeze_enabled() {
            let old_score = self.score as i32;
            if let Some(_close) = &self.close_overlap {
                self.score += self.close_points_per_sec * delta as f32;
            } else if let Some(_far) = &self.far_overlap {
                self.score += self.far_points_per_sec * delta as f32;
            }
            let new = self.score as i32;
            if old_score != new {
                self.base_mut()
                    .emit_signal("score_updated".into(), &[new.to_variant()]);
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if self.base().is_freeze_enabled() {
            return;
        }

        let force = self.forward_force * delta as f32 * -self.base().get_global_basis().col_c();
        self.base_mut().apply_force(force);
    }
}

#[godot_api]
impl Player {
    #[signal]
    fn player_reset() {}

    #[signal]
    fn score_updated(new_score: i32) {}

    // func is necessary to be callable from gd / signals
    #[func]
    fn on_collision(&mut self, node: Gd<Node>) {
        godot_print!("Collided with {}", node.get_name());
        self.base_mut().set_freeze_enabled(true);
        self.base_mut().set_visible(false);

        if let Some(explosion_scene) = self.get_explosion_scene() {
            let mut explosion_node = explosion_scene.instantiate_as::<Node3D>();
            if let Some(tree) = self.base().get_tree() {
                if let Some(mut root) = tree.get_root() {
                    explosion_node.set_position(self.base().get_global_position());
                    root.add_child(explosion_node);
                }
            }
        }

        if let Some(mut timer) = self.get_respawn_timer() {
            timer.start();
        }

        self.is_dead = true;
    }

    #[func]
    pub fn on_timer_timeout(&mut self) {
        godot_print!("Respawning");
        self.base_mut().set_global_position(Vector3::ZERO);
        self.base_mut().set_freeze_enabled(false);
        self.base_mut().set_visible(true);
        self.base_mut().emit_signal("player_reset".into(), &[]);

        self.score = 0.0;
        self.base_mut()
            .emit_signal("score_updated".into(), &[0_i32.to_variant()]);

        self.is_dead = false;
    }

    #[func]
    pub fn on_close_body_overlap(&mut self, node: Gd<Node>) {
        self.close_overlap = Some(node);
    }

    #[func]
    pub fn on_close_body_exit(&mut self, _node: Gd<Node>) {
        self.close_overlap = None;
    }

    #[func]
    pub fn on_far_body_overlap(&mut self, node: Gd<Node>) {
        self.far_overlap = Some(node);
    }

    #[func]
    pub fn on_far_body_exit(&mut self, _node: Gd<Node>) {
        self.far_overlap = None;
    }
}
