use crate::linkl::linkl_error::Res;

use super::linkl_error::LinklError;

#[derive(Clone, Debug)]
pub struct EthernetFrame {
    pub dst: u64,       // 6 bytes
    pub src: u64,       // 6 bytes
    pub ethertype: u16, // 2 bytes
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn new(dst: u64, src: u64, ethertype: u16, payload: Vec<u8>) -> EthernetFrame {
        EthernetFrame {
            dst,
            src,
            ethertype,
            payload,
        }
    }

    pub fn decode(xs: &Vec<u8>) -> Res<EthernetFrame> {
        if xs.len() < 8 + 6 + 6 + 2 {
            return Err(LinklError::NotEnoughBytes);
        }
        for i in 0..7 {
            if xs[i] != 0xAA {
                // 10101010 = 0xAA
                return Err(LinklError::InvalidBytes {
                    msg: "bad preamble".to_string(),
                });
            }
        }
        if xs[7] != 0xAB {
            // 10101011 = 0xAB
            return Err(LinklError::InvalidBytes {
                msg: "bad preamble".to_string(),
            });
        }
        let dst = read_6bytes(xs, 8);
        let src = read_6bytes(xs, 8 + 6);
        let ty = read_2bytes(xs, 8 + 6 + 6);
        if ty > 0x05DC {
            return Err(LinklError::InvalidBytes {
                msg: format!("unsupported ethernet type: {}", ty),
            });
        } else {
            // type is length
            let len = ty as usize;
            if xs.len() < 8 + 6 + 6 + 2 + len {
                return Err(LinklError::NotEnoughBytes);
            }
            let payload = Vec::from(&xs[(8 + 6 + 6 + 2)..(8 + 6 + 6 + 2 + len)]);
            Ok(EthernetFrame {
                dst,
                src,
                ethertype: ty,
                payload,
            })
        }
    }

    pub fn encode(frame: &EthernetFrame) -> Vec<u8> {
        let et = split_2bytes(frame.ethertype);
        let dst = split_6bytes(frame.dst);
        let src = split_6bytes(frame.src);
        let mut xs = vec![
            0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAB, // preamble
            dst[0], dst[1], dst[2], dst[3], dst[4], dst[5], src[0], src[1], src[2], src[3], src[4],
            src[5], et[0], et[1],
        ];
        // FIXME: avoid clone
        xs.append(&mut frame.payload.clone());
        xs
    }
}

fn read_2bytes(xs: &Vec<u8>, offset: usize) -> u16 {
    (xs[offset] as u16) << 8 | (xs[offset + 1] as u16)
}

fn read_6bytes(xs: &Vec<u8>, offset: usize) -> u64 {
    (xs[offset] as u64) << 40
        | (xs[offset + 1] as u64) << 32
        | (xs[offset + 2] as u64) << 24
        | (xs[offset + 3] as u64) << 16
        | (xs[offset + 4] as u64) << 8
        | (xs[offset + 5] as u64)
}

fn split_2bytes(x: u16) -> [u8; 2] {
    [(x >> 8) as u8, (x & 0xFF) as u8]
}

fn split_6bytes(x: u64) -> [u8; 6] {
    [
        (x >> 40) as u8,
        ((x >> 32) & 0xFF) as u8,
        ((x >> 24) & 0xFF) as u8,
        ((x >> 16) & 0xFF) as u8,
        ((x >> 8) & 0xFF) as u8,
        (x & 0xFF) as u8,
    ]
}

pub fn run_linkl_sample() {
    println!("link_sample start");
    let xs = vec![
        0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAB, // preamble
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // dst
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02, // src
        0x00, 0x04, // type
        0x01, 0x02, 0x03, 0x04, // payload
    ];
    let frame = EthernetFrame::decode(&xs).unwrap();
    println!("dst: {:012X}", frame.dst);
    println!("src: {:012X}", frame.src);
    println!("type: {:04X}", frame.ethertype);
    println!("payload: {:?}", frame.payload);
}
