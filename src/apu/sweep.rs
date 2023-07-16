#[derive(Default)]
pub struct Sweep {
    pub enabled: bool,
    pub reload: bool,
    pub negate: bool,
    pub timer: u8,
    pub counter: u8,
    pub shift: u8,
}
