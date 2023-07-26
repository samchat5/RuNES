#[derive(Default)]
pub struct Sweep {
    pub enabled: bool,
    pub negate: bool,
    pub divider: u8,
    pub shift: u8,
    pub target_period: u32,
    pub period: u8,
    pub reload: bool,
}