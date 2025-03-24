use crate::sys::*;

/// Range struct to represent `bladerf_range`
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

impl Range {
    pub fn contains(&self, query: impl Into<u64>) -> bool {
        let steps = (query.into() as f64 - self.min) / self.step;
        steps % 1.0 < 1e-8
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:.0}..{:.0} (step {:.0})",
            self.min, self.max, self.step,
        ))
    }
}

impl From<&bladerf_range> for Range {
    fn from(range: &bladerf_range) -> Self {
        Self {
            min: range.min as f64 * range.scale as f64,
            max: range.max as f64 * range.scale as f64,
            step: range.step as f64 * range.scale as f64,
        }
    }
}
