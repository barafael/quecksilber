use rand::Rng;

/// A value that drifts randomly within a bounded range, producing smooth animation.
pub struct RandomWalk {
    value: f32,
    min: f32,
    max: f32,
    step: f32,
}

impl RandomWalk {
    pub fn new(initial: f32, min: f32, max: f32, step: f32) -> Self {
        Self {
            value: initial,
            min,
            max,
            step,
        }
    }

    /// Advance the walk by one tick, returning the new value.
    pub fn tick(&mut self) -> f32 {
        let mut rng = rand::rng();
        let delta: f32 = rng.random_range(-self.step..=self.step);
        self.value = (self.value + delta).clamp(self.min, self.max);
        self.value
    }
}
