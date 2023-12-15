// use std::collections::VecDeque;
// use crate::physl::{device::{DeviceOperation, DeviceContext}, physl_error::Res};

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
// struct Ether {}
// impl DeviceOperation for Ether {
//     fn apply(&mut self, ctx: &DeviceContext, port: usize, rbuf: &VecDeque<u8>) -> Res<Vec<(usize, Vec<u8>)>> {
//         let a = rbuf.as_slices();
//         // let x = EthernetFrame::decode(&(rbuf as Vec<u8>));
//         Ok(vec![])
//     }
// }

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
