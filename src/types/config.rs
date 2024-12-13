use super::ModuleConfig;

/// Combined RX and TX config
pub struct Config {
    pub tx: ModuleConfig,
    pub rx: ModuleConfig,
}
