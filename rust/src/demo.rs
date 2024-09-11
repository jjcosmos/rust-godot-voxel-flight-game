use godot::classes::{ISprite2D, Sprite2D, Time};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Sprite2D)]
struct Demo {
    speed: f64,
    angular_speed: f64,

    base: Base<Sprite2D>,
}

#[godot_api]
impl ISprite2D for Demo {
    fn init(base: Base<Sprite2D>) -> Self {
        godot_print!("Hello World");

        Self {
            speed: 400.0,
            angular_speed: std::f64::consts::PI,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let radians = (self.angular_speed * delta) as f32;
        self.base_mut().rotate(radians);

        let rotation = self.base().get_rotation();
        let velocity = Vector2::UP.rotated(rotation) * self.speed as f32;
        self.base_mut().translate(
            velocity
                * delta as f32
                * ((Time::singleton().get_ticks_msec() as f64 / 1000.0) as f32).sin(),
        )
    }
}

#[godot_api]
impl Demo {
    #[func]
    fn increase_speed(&mut self, amount: f64) {
        self.speed += amount;
        self.base_mut().emit_signal("speed_increased".into(), &[]);
    }

    #[signal]
    fn speed_increased();
}
