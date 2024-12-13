/// BladeRF module config object
#[derive(Clone, Debug)]
pub struct ModuleConfig {
    pub frequency: u64,
    pub sample_rate: u32,
    pub bandwidth: u32,
    /// Set overall system gain
    pub gain: i32,
}
