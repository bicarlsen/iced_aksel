use std::ops::{Add, Div, Mul, Sub};

use aksel::Float;

/// Defines how a dimension should be interpreted by `Chart`.
#[derive(Clone, Copy, Debug)]
pub enum Measure<D> {
    /// Fixed size in screen pixels (e.g., "10px wide").
    /// Does not scale when zooming the chart.
    Screen(f32),

    /// Size in chart data units (e.g., "5.0 units on the X axis").
    /// Scales when zooming the chart.
    Plot(D),
}

// 1. Implement Multiplication: Length * Number
impl<D: Float> Mul<D> for Measure<D> {
    type Output = Self;

    fn mul(self, scalar: D) -> Self {
        match self {
            // Convert scalar to f32 for Screen pixels
            Self::Screen(px) => Self::Screen(px * scalar.to_f32().unwrap()),
            // Direct multiplication for Plot units
            Self::Plot(val) => Self::Plot(val * scalar),
        }
    }
}

// 2. Implement Division: Length / Number
impl<D: Float> Div<D> for Measure<D> {
    type Output = Self;

    fn div(self, scalar: D) -> Self {
        match self {
            Self::Screen(px) => Self::Screen(px / scalar.to_f32().unwrap()),
            Self::Plot(val) => Self::Plot(val / scalar),
        }
    }
}

// --- Addition (Length + Number) ---
impl<D: Float> Add<D> for Measure<D> {
    type Output = Self;

    fn add(self, scalar: D) -> Self {
        match self {
            // Add to pixels (convert scalar to f32)
            Self::Screen(px) => Self::Screen(px + scalar.to_f32().unwrap()),
            // Add to plot units
            Self::Plot(val) => Self::Plot(val + scalar),
        }
    }
}

// --- Subtraction (Length - Number) ---
impl<D: Float> Sub<D> for Measure<D> {
    type Output = Self;

    fn sub(self, scalar: D) -> Self {
        match self {
            Self::Screen(px) => Self::Screen(px - scalar.to_f32().unwrap()),
            Self::Plot(val) => Self::Plot(val - scalar),
        }
    }
}
