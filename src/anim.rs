use iced::Color;

/// A value that interpolates toward a target using spring dynamics.
///
/// Uses a damped harmonic oscillator model:
/// - `stiffness` controls how fast the spring pulls toward the target (higher = snappier).
/// - `damping` controls energy dissipation (1.0 = critically damped, <1.0 = oscillates, >1.0 = overdamped).
#[derive(Debug, Clone, Copy)]
pub struct Spring {
    value: f32,
    velocity: f32,
    target: f32,
    stiffness: f32,
    damping: f32,
}

/// Default spring parameters: critically damped, medium stiffness.
const DEFAULT_STIFFNESS: f32 = 180.0;
const DEFAULT_DAMPING: f32 = 1.0;
const SETTLE_THRESHOLD: f32 = 0.001;
const VELOCITY_THRESHOLD: f32 = 0.01;

impl Spring {
    /// Creates a new spring at rest at the given initial value.
    /// Uses critically damped defaults.
    #[must_use]
    pub fn new(initial: f32) -> Self {
        Self::with_params(initial, DEFAULT_STIFFNESS, DEFAULT_DAMPING)
    }

    /// Creates a new spring with custom stiffness and damping.
    ///
    /// - `stiffness`: spring constant (typically 50–500).
    /// - `damping`: damping ratio (1.0 = critically damped).
    #[must_use]
    pub fn with_params(initial: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target: initial,
            stiffness,
            damping,
        }
    }

    /// Sets a new target value for the spring to animate toward.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Returns the current target.
    #[must_use]
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Advances the spring simulation by `dt` seconds.
    ///
    /// Uses semi-implicit Euler integration of the damped harmonic oscillator:
    ///   acceleration = -stiffness * displacement - damping_force * velocity
    pub fn tick(&mut self, dt: f32) {
        let displacement = self.value - self.target;
        // Critical damping coefficient: 2 * sqrt(stiffness)
        let damping_force = 2.0 * self.damping * self.stiffness.sqrt();
        let acceleration = -self.stiffness * displacement - damping_force * self.velocity;
        self.velocity += acceleration * dt;
        self.value += self.velocity * dt;
    }

    /// Returns the current animated value.
    #[must_use]
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Returns `true` when the spring has effectively reached its target.
    #[must_use]
    pub fn is_settled(&self) -> bool {
        let displacement = (self.value - self.target).abs();
        let speed = self.velocity.abs();
        displacement < SETTLE_THRESHOLD && speed < VELOCITY_THRESHOLD
    }

    /// Instantly snaps to the target without animation.
    pub fn snap(&mut self) {
        self.value = self.target;
        self.velocity = 0.0;
    }
}

/// A color that interpolates toward a target using exponential decay.
#[derive(Debug, Clone, Copy)]
pub struct ColorTransition {
    pub(crate) current: Color,
    target: Color,
    speed: f32,
}

const DEFAULT_COLOR_SPEED: f32 = 8.0;

impl ColorTransition {
    /// Creates a new color transition starting at the given color.
    #[must_use]
    pub fn new(initial: Color) -> Self {
        Self::with_speed(initial, DEFAULT_COLOR_SPEED)
    }

    /// Creates a new color transition with a custom speed (higher = faster).
    #[must_use]
    pub fn with_speed(initial: Color, speed: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            speed,
        }
    }

    /// Sets a new target color to transition toward.
    pub fn set_target(&mut self, target: Color) {
        self.target = target;
    }

    /// Advances the transition by `dt` seconds using exponential decay.
    pub fn tick(&mut self, dt: f32) {
        let t = 1.0 - (-self.speed * dt).exp();
        self.current = Color {
            r: lerp(self.current.r, self.target.r, t),
            g: lerp(self.current.g, self.target.g, t),
            b: lerp(self.current.b, self.target.b, t),
            a: lerp(self.current.a, self.target.a, t),
        };
    }

    /// Returns the current interpolated color.
    #[must_use]
    pub fn value(&self) -> Color {
        self.current
    }

    /// Returns `true` when the color is close enough to the target.
    #[must_use]
    pub fn is_settled(&self) -> bool {
        color_distance(self.current, self.target) < 0.005
    }

    /// Instantly snaps to the target color without animation.
    pub fn snap(&mut self) {
        self.current = self.target;
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn color_distance(a: Color, b: Color) -> f32 {
    let dr = a.r - b.r;
    let dg = a.g - b.g;
    let db = a.b - b.b;
    let da = a.a - b.a;
    (dr * dr + dg * dg + db * db + da * da).sqrt()
}

/// Returns a sinusoidal "breathe" pulse value in [0.0, 1.0] for the given time in seconds.
#[must_use]
pub fn breathe(time_secs: f32, period_secs: f32) -> f32 {
    let phase = (time_secs / period_secs) * std::f32::consts::TAU;
    (phase.sin() + 1.0) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_starts_at_initial_value() {
        let spring = Spring::new(5.0);
        assert!((spring.value() - 5.0).abs() < f32::EPSILON);
        assert!(spring.is_settled());
    }

    #[test]
    fn spring_moves_toward_target() {
        let mut spring = Spring::new(0.0);
        spring.set_target(10.0);
        assert!(!spring.is_settled());

        // Simulate 1 second at 60fps
        for _ in 0..60 {
            spring.tick(1.0 / 60.0);
        }

        // Should be close to target after 1 second with critically damped spring
        assert!(
            (spring.value() - 10.0).abs() < 0.5,
            "Spring should be near target, got {}",
            spring.value()
        );
    }

    #[test]
    fn spring_settles_after_sufficient_time() {
        let mut spring = Spring::new(0.0);
        spring.set_target(100.0);

        // Simulate 3 seconds
        for _ in 0..180 {
            spring.tick(1.0 / 60.0);
        }

        assert!(
            spring.is_settled(),
            "Spring should be settled after 3s, value={} target={}",
            spring.value(),
            spring.target()
        );
    }

    #[test]
    fn spring_snap_jumps_to_target() {
        let mut spring = Spring::new(0.0);
        spring.set_target(42.0);
        spring.snap();
        assert!((spring.value() - 42.0).abs() < f32::EPSILON);
        assert!(spring.is_settled());
    }

    #[test]
    fn underdamped_spring_oscillates() {
        let mut spring = Spring::with_params(0.0, 200.0, 0.3);
        spring.set_target(10.0);

        let mut crossed_target = false;
        for _ in 0..120 {
            spring.tick(1.0 / 60.0);
            if spring.value() > 10.0 {
                crossed_target = true;
            }
        }

        assert!(
            crossed_target,
            "Underdamped spring should overshoot the target"
        );
    }

    #[test]
    fn color_transition_starts_at_initial() {
        let ct = ColorTransition::new(Color::WHITE);
        assert_eq!(ct.value(), Color::WHITE);
        assert!(ct.is_settled());
    }

    #[test]
    fn color_transition_moves_toward_target() {
        let mut ct = ColorTransition::new(Color::BLACK);
        ct.set_target(Color::WHITE);

        for _ in 0..60 {
            ct.tick(1.0 / 60.0);
        }

        let v = ct.value();
        assert!(v.r > 0.9, "Red channel should be near 1.0, got {}", v.r);
    }

    #[test]
    fn color_transition_snap() {
        let mut ct = ColorTransition::new(Color::BLACK);
        ct.set_target(Color::from_rgb(0.5, 0.3, 0.7));
        ct.snap();
        assert!(ct.is_settled());
    }

    #[test]
    fn breathe_oscillates_between_zero_and_one() {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for i in 0..100 {
            let t = i as f32 * 0.02; // 0..2 seconds
            let v = breathe(t, 1.0);
            min = min.min(v);
            max = max.max(v);
        }
        assert!(min < 0.05, "Breathe should reach near 0, got {min}");
        assert!(max > 0.95, "Breathe should reach near 1, got {max}");
    }
}
