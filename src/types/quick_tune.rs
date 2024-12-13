#[derive(Clone, Debug)]
#[repr(C)]
pub struct QuickTune {
    pub freqsel: u8,
    pub vcocap: u8,
    pub nint: u16,
    pub nfrac: u32,
    pub flags: u8,
}
