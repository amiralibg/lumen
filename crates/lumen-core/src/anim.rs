//! Springs and fades.
//!
//! Springs are solved in closed form (damped harmonic oscillator), so they
//! are exact and frame-rate independent, keep velocity when retargeted, and
//! settle deterministically.

use crate::id::Id;
use crate::FastMap;

/// Spring constants with mass 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringParams {
    pub stiffness: f32,
    pub damping: f32,
}

impl SpringParams {
    /// From Apple-style response (seconds) and damping ratio.
    pub fn from_response(response: f32, damping_ratio: f32) -> Self {
        let w = std::f32::consts::TAU / response;
        SpringParams { stiffness: w * w, damping: 2.0 * damping_ratio * w }
    }
    pub fn damping_ratio(&self) -> f32 {
        self.damping / (2.0 * self.stiffness.sqrt())
    }
}

/// Position and velocity of a spring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
}

impl Spring {
    pub fn new(v: f32) -> Self {
        Spring { value: v, velocity: 0.0, target: v }
    }

    pub fn is_at_rest(&self, eps: f32) -> bool {
        (self.value - self.target).abs() < eps && self.velocity.abs() < 0.01
    }

    /// Advance by `dt` seconds toward `self.target`.
    pub fn step(&mut self, p: SpringParams, dt: f32, eps: f32) {
        if dt <= 0.0 {
            return;
        }
        let y0 = self.value - self.target;
        let v0 = self.velocity;
        let w0 = p.stiffness.max(1e-6).sqrt();
        let zeta = p.damping / (2.0 * w0);
        let (y, v);
        if (zeta - 1.0).abs() < 1e-3 {
            let b = v0 + w0 * y0;
            let e = (-w0 * dt).exp();
            y = e * (y0 + b * dt);
            v = e * (b - w0 * (y0 + b * dt));
        } else if zeta < 1.0 {
            let wd = w0 * (1.0 - zeta * zeta).sqrt();
            let e = (-zeta * w0 * dt).exp();
            let b = (v0 + zeta * w0 * y0) / wd;
            let (s, c) = (wd * dt).sin_cos();
            y = e * (y0 * c + b * s);
            v = e * (-zeta * w0 * (y0 * c + b * s) + (-y0 * wd * s + b * wd * c));
        } else {
            let sq = (zeta * zeta - 1.0).sqrt();
            let r1 = -w0 * (zeta - sq);
            let r2 = -w0 * (zeta + sq);
            let c1 = (v0 - r2 * y0) / (r1 - r2);
            let c2 = y0 - c1;
            let (e1, e2) = ((r1 * dt).exp(), (r2 * dt).exp());
            y = c1 * e1 + c2 * e2;
            v = c1 * r1 * e1 + c2 * r2 * e2;
        }
        self.value = self.target + y;
        self.velocity = v;
        if self.is_at_rest(eps) {
            self.value = self.target;
            self.velocity = 0.0;
        }
    }
}

/// Easing curves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ease {
    Linear,
    /// `cubic-bezier(0.4, 0, 0.2, 1)`.
    Standard,
    /// Quadratic ease-out.
    Out,
}

impl Ease {
    pub fn apply(self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        match self {
            Ease::Linear => t,
            Ease::Out => 1.0 - (1.0 - t) * (1.0 - t),
            Ease::Standard => cubic_bezier(0.4, 0.0, 0.2, 1.0, t),
        }
    }
}

fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let bez = |a: f32, b: f32, t: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    };
    // Solve x(t) = x by bisection; precision is plenty for UI.
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..20 {
        let mid = (lo + hi) / 2.0;
        if bez(x1, x2, mid) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    bez(y1, y2, (lo + hi) / 2.0)
}

#[derive(Clone, Copy, Debug)]
struct Fade {
    /// Linear progress 0..1.
    value: f32,
    rising: bool,
}

/// Per-widget animation state keyed by `(Id, slot)`.
#[derive(Default)]
pub struct AnimStore {
    springs: FastMap<(Id, u32), (Spring, u64)>,
    fades: FastMap<(Id, u32), (Fade, u64)>,
    frame: u64,
    dt: f32,
    active: bool,
    /// Multiplies time (1.0 normal, 2.0 = half speed).
    pub time_scale: f32,
    /// Jump springs to their target and cap fades (reduced motion).
    pub reduced_motion: bool,
}

impl AnimStore {
    pub fn new() -> Self {
        Self { time_scale: 1.0, ..Default::default() }
    }

    /// Begin a frame that advances time by `dt` seconds.
    pub fn begin_frame(&mut self, dt: f32) {
        self.frame += 1;
        self.dt = (dt / self.time_scale.max(0.01)).clamp(0.0, 0.1);
        self.active = false;
    }

    /// Drop state for widgets that were not seen for a few frames.
    pub fn end_frame(&mut self) {
        let f = self.frame;
        self.springs.retain(|_, (_, seen)| f - *seen < 4);
        self.fades.retain(|_, (_, seen)| f - *seen < 4);
    }

    /// True if any animation needs another frame.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Animate toward `target` with a spring; returns the current value.
    /// The first call for a key starts at `target` (no entrance animation).
    pub fn spring(&mut self, id: Id, slot: u32, target: f32, p: SpringParams) -> f32 {
        self.spring_from(id, slot, target, target, p)
    }

    /// Like [`spring`](Self::spring) but the first call starts at `initial`.
    pub fn spring_from(&mut self, id: Id, slot: u32, initial: f32, target: f32, p: SpringParams) -> f32 {
        let frame = self.frame;
        let dt = self.dt;
        let reduced = self.reduced_motion;
        let mut inserted = false;
        let entry = self.springs.entry((id, slot)).or_insert_with(|| {
            inserted = true;
            (Spring::new(initial), frame)
        });
        let (s, seen) = entry;
        if *seen != frame || inserted {
            *seen = frame;
            s.target = target;
            if reduced {
                *s = Spring::new(target);
            } else {
                s.step(p, dt, 0.001);
            }
        } else {
            s.target = target;
        }
        if !s.is_at_rest(0.001) {
            self.active = true;
        }
        s.value
    }

    /// Set a spring's value directly (e.g. while dragging) and zero velocity.
    pub fn set_spring(&mut self, id: Id, slot: u32, value: f32) {
        self.springs.insert((id, slot), (Spring::new(value), self.frame));
    }

    /// Current spring state, if any.
    pub fn spring_state(&self, id: Id, slot: u32) -> Option<Spring> {
        self.springs.get(&(id, slot)).map(|(s, _)| *s)
    }

    /// Fade a 0..1 value toward `on` linearly over `in_s` / `out_s` seconds.
    /// The returned value is eased with `ease` in either direction.
    pub fn fade(&mut self, id: Id, slot: u32, on: bool, in_s: f32, out_s: f32, ease: Ease) -> f32 {
        let frame = self.frame;
        let dt = self.dt;
        let reduced = self.reduced_motion;
        let entry = self.fades.entry((id, slot)).or_insert_with(|| (Fade { value: if on { 1.0 } else { 0.0 }, rising: on }, frame));
        let (f, seen) = entry;
        if *seen != frame {
            *seen = frame;
            let mut d = if on { in_s } else { out_s };
            if reduced {
                d = d.min(0.1);
            }
            let step = if d <= 0.0 { 1.0 } else { dt / d };
            f.rising = on;
            f.value = if on { (f.value + step).min(1.0) } else { (f.value - step).max(0.0) };
        }
        let target = if on { 1.0 } else { 0.0 };
        if f.value != target {
            self.active = true;
        }
        let v = f.value;
        if f.rising {
            ease.apply(v)
        } else {
            1.0 - ease.apply(1.0 - v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::springs::*;

    fn run(p: SpringParams, dt: f32, total: f32) -> (Spring, f32) {
        let mut s = Spring::new(0.0);
        s.target = 100.0;
        let mut peak = 0.0f32;
        let mut t = 0.0;
        while t < total {
            s.step(p, dt, 0.01);
            peak = peak.max(s.value);
            t += dt;
        }
        (s, peak)
    }

    #[test]
    fn critically_damped_springs_never_overshoot() {
        for p in [SNAPPY, SMOOTH, GENTLE] {
            let (s, peak) = run(p, 1.0 / 60.0, 2.0);
            assert!(peak <= 100.0 + 1e-3, "{p:?} peaked at {peak}");
            assert_eq!(s.value, 100.0, "{p:?} should settle");
        }
    }

    #[test]
    fn bouncy_overshoots_about_five_percent() {
        let (_, peak) = run(BOUNCY, 1.0 / 240.0, 1.0);
        let over = peak - 100.0;
        assert!((3.5..6.0).contains(&over), "overshoot {over}");
    }

    #[test]
    fn frame_rate_independent() {
        let (a, _) = run(SMOOTH, 1.0 / 30.0, 0.2);
        let (b, _) = run(SMOOTH, 1.0 / 240.0, 0.2);
        // Same elapsed time (within one step of the coarser clock).
        assert!((a.value - b.value).abs() < 3.0, "{} vs {}", a.value, b.value);
        let mut c = Spring::new(0.0);
        c.target = 100.0;
        c.step(SMOOTH, 0.2, 0.0);
        let mut d = Spring::new(0.0);
        d.target = 100.0;
        for _ in 0..48 {
            d.step(SMOOTH, 0.2 / 48.0, 0.0);
        }
        assert!((c.value - d.value).abs() < 1e-2, "{} vs {}", c.value, d.value);
    }

    #[test]
    fn response_and_ratio_match_design() {
        let p = SpringParams::from_response(0.20, 1.0);
        assert!((p.stiffness - SNAPPY.stiffness).abs() < 1.0);
        assert!((SNAPPY.damping_ratio() - 1.0).abs() < 0.01);
        assert!((BOUNCY.damping_ratio() - 0.70).abs() < 0.01);
    }

    #[test]
    fn retarget_keeps_velocity() {
        let mut s = Spring::new(0.0);
        s.target = 100.0;
        s.step(SNAPPY, 0.05, 0.01);
        let v = s.velocity;
        s.target = 0.0;
        assert_eq!(s.velocity, v);
        s.step(SNAPPY, 0.001, 0.01);
        assert!(s.velocity > 0.0, "still moving forward right after retarget");
    }

    #[test]
    fn store_spring_starts_at_target_and_reports_activity() {
        let mut st = AnimStore::new();
        let id = Id::new("x");
        st.begin_frame(1.0 / 60.0);
        assert_eq!(st.spring(id, 0, 5.0, SNAPPY), 5.0);
        assert!(!st.is_active());
        st.begin_frame(1.0 / 60.0);
        let v = st.spring(id, 0, 10.0, SNAPPY);
        assert!(v > 5.0 && v < 10.0);
        assert!(st.is_active());
    }

    #[test]
    fn fade_timing() {
        let mut st = AnimStore::new();
        let id = Id::new("f");
        st.begin_frame(0.0);
        assert_eq!(st.fade(id, 0, false, 0.06, 0.16, Ease::Linear), 0.0);
        st.begin_frame(0.03);
        let v = st.fade(id, 0, true, 0.06, 0.16, Ease::Linear);
        assert!((v - 0.5).abs() < 1e-4);
        st.begin_frame(0.03);
        assert_eq!(st.fade(id, 0, true, 0.06, 0.16, Ease::Linear), 1.0);
        st.begin_frame(0.08);
        let v = st.fade(id, 0, false, 0.06, 0.16, Ease::Linear);
        assert!((v - 0.5).abs() < 1e-4);
    }

    #[test]
    fn reduced_motion_jumps() {
        let mut st = AnimStore::new();
        st.reduced_motion = true;
        let id = Id::new("r");
        st.begin_frame(1.0 / 60.0);
        st.spring(id, 0, 0.0, SNAPPY);
        st.begin_frame(1.0 / 60.0);
        assert_eq!(st.spring(id, 0, 50.0, SNAPPY), 50.0);
    }

    #[test]
    fn standard_ease_endpoints() {
        assert_eq!(Ease::Standard.apply(0.0), 0.0);
        assert!((Ease::Standard.apply(1.0) - 1.0).abs() < 1e-4);
        assert!(Ease::Standard.apply(0.5) > 0.5);
    }
}
