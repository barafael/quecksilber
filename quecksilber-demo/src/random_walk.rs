use rand::Rng;

/// A value that smoothly animates toward random target positions within a bounded range.
///
/// Each tick lerps the current value toward the target. When the value is close
/// enough to the target, a new random target is chosen.
pub struct RandomWalk {
    value: f32,
    target: f32,
    min: f32,
    max: f32,
    /// Fraction of the remaining distance to cover per tick (0..1).
    speed: f32,
    /// Distance threshold to consider "arrived" and pick a new target.
    threshold: f32,
}

impl RandomWalk {
    pub fn new(initial: f32, min: f32, max: f32, speed: f32) -> Self {
        let range = max - min;
        let mut s = Self {
            value: initial,
            target: initial,
            min,
            max,
            speed,
            threshold: range * 0.01,
        };
        s.pick_target();
        s
    }

    fn pick_target(&mut self) {
        let mut rng = rand::rng();
        self.target = rng.random_range(self.min..=self.max);
    }

    /// Advance one tick, returning the new value.
    pub fn tick(&mut self) -> f32 {
        let diff = self.target - self.value;
        if diff.abs() < self.threshold {
            self.pick_target();
        }
        self.value += diff * self.speed;
        self.value
    }
}
