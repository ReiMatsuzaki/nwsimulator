
pub fn read_2bytes(xs: &Vec<u8>, offset: usize) -> u16 {
    (xs[offset] as u16) << 8 | (xs[offset + 1] as u16)
}

pub fn read_6bytes(xs: &Vec<u8>, offset: usize) -> u64 {
    (xs[offset] as u64) << 40
        | (xs[offset + 1] as u64) << 32
        | (xs[offset + 2] as u64) << 24
        | (xs[offset + 3] as u64) << 16
        | (xs[offset + 4] as u64) << 8
        | (xs[offset + 5] as u64)
}

pub fn split_2bytes(x: u16) -> [u8; 2] {
    [(x >> 8) as u8, (x & 0xFF) as u8]
}

pub fn split_6bytes(x: u64) -> [u8; 6] {
    [
        (x >> 40) as u8,
        ((x >> 32) & 0xFF) as u8,
        ((x >> 24) & 0xFF) as u8,
        ((x >> 16) & 0xFF) as u8,
        ((x >> 8) & 0xFF) as u8,
        (x & 0xFF) as u8,
    ]
}

