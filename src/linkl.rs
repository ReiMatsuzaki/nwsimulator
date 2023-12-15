struct EthernetFrame {
    dst: u64, // 6 bytes
    src: u64, // 6 bytes
    ethertype: u16, // 2 bytes
    payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn decode(xs: &Vec<u8>) -> Option<EthernetFrame> {
        for i in 0..7 {
            if xs[i] != 0xAA { // 10101010 = 0xAA
                return None;
            }
        }
        if xs[7] != 0xAB { // 10101011 = 0xAB
            return None;
        }
        let mut dst: u64 = 0;
        for i in 0..6 {
            dst = dst | (xs[i + 8] as u64) << (5 - i) * 8;
        }
        let dst = read_6bytes(xs, 8);
        let src = read_6bytes(xs, 8+6);
        let ty = read_2bytes(xs, 8+6+6);
        if ty <= 0x05DC {
            let len = ty as usize;
            let x= &xs[(8+6+6+2)..(8+6+6+2+len)];
            let payload = Vec::from(x);
            Some(EthernetFrame {
                dst,
                src,
                ethertype: ty,
                payload,
            })
        } else {
            None
        }
    }
}

fn read_2bytes(xs: &Vec<u8>, offset: usize) -> u16 {
    (xs[offset] as u16) << 8 | (xs[offset + 1] as u16)
}

fn read_6bytes(xs: &Vec<u8>, offset: usize) -> u64 {
    (xs[offset] as u64) << 40 | (xs[offset + 1] as u64) << 32 | (xs[offset + 2] as u64) << 24 | (xs[offset + 3] as u64) << 16 | (xs[offset + 4] as u64) << 8 | (xs[offset + 5] as u64)
}