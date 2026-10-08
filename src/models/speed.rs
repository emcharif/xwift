const G: f64 = 9.81;
const RHO: f64 = 1.225; // air density, kg/m^3
const DRIVETRAIN_EFF: f64 = 0.975;

// Tune these to you:
pub const CDA: f64 = 0.32; // drag area, m^2 (lower = more aero position)
pub const CRR: f64 = 0.004; // rolling resistance
pub const MASS_KG: f64 = 75.0; // rider + bike

/// Flat-road speed in km/h that this power would produce (no wind).
pub fn speed_kmh(power_w: f64) -> f64 {
    if power_w <= 0.0 {
        return 0.0;
    }
    let p = power_w * DRIVETRAIN_EFF;
    let needed = |v: f64| 0.5 * RHO * CDA * v * v * v + CRR * MASS_KG * G * v;

    // Power needed grows with speed, so bisection finds the speed where needed == p.
    let (mut lo, mut hi) = (0.0, 30.0);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if needed(mid) < p {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0 * 3.6
}