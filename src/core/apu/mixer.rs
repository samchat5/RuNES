use lazy_static::lazy_static;

lazy_static! {
    static ref PULSE_TABLE: [f32; 31] = (0u16..=30u16)
        .into_iter()
        .map(|i| {
            if i == 0 {
                0.0
            } else {
                95.52 / (8_128.0 / (f32::from(i)) + 100.0)
            }
        })
        .collect::<Vec<f32>>()
        .as_slice()
        .try_into()
        .unwrap();
    static ref TND_TABLE: [f32; 203] = (0u16..=202u16)
        .into_iter()
        .map(|i| {
            if i == 0 {
                0.0
            } else {
                163.67 / (24_329.0 / (f32::from(i)) + 100.0)
            }
        })
        .collect::<Vec<f32>>()
        .as_slice()
        .try_into()
        .unwrap();
}

pub(super) fn mixer_value(pulse1: u8, pulse2: u8, _triangle: f32, _noise: f32, _dmc: f32) -> f32 {
    let mut pulse_idx = (pulse1 + pulse2) as usize;
    if pulse_idx > PULSE_TABLE.len() {
        pulse_idx %= PULSE_TABLE.len();
    }
    PULSE_TABLE[pulse_idx]
}
