use std::fmt;

use crate::physl::{device::{DeviceOperation, DeviceContext, Device}, physl_error::PhyslError, host::Host, network::Network};

#[derive(Debug)]
pub enum LinklError {
    NotEnoughBytes,
    InvalidBytes {msg: String},
}

impl fmt::Display for LinklError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinklError::NotEnoughBytes =>
                write!(f, "Not enough bytes"),
            LinklError::InvalidBytes {msg} =>
                write!(f, "Invalid Bytes:. {}", msg),
        }
    }
}

type Res<T> = Result<T, LinklError>;

struct EthernetFrame {
    dst: u64, // 6 bytes
    src: u64, // 6 bytes
    ethertype: u16, // 2 bytes
    payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn decode(xs: &Vec<u8>) -> Res<EthernetFrame> {
        if xs.len() < 8 + 6 + 6 + 2 {
            return Err(LinklError::NotEnoughBytes);
        }
        for i in 0..7 {
            if xs[i] != 0xAA { // 10101010 = 0xAA
                return Err(LinklError::InvalidBytes{
                    msg: "bad preamble".to_string()
                });
            }
        }
        if xs[7] != 0xAB { // 10101011 = 0xAB
            return Err(LinklError::InvalidBytes{
                msg: "bad preamble".to_string()
            })
        }
        let dst = read_6bytes(xs, 8);
        let src = read_6bytes(xs, 8+6);
        let ty = read_2bytes(xs, 8+6+6);
        if ty > 0x05DC {
            return Err(LinklError::InvalidBytes{
                msg: format!("unsupported ethernet type: {}", ty)
            })
        } else {
            // type is length
            let len = ty as usize;
            if xs.len() < 8 + 6 + 6 + 2 + len {
                return Err(LinklError::NotEnoughBytes);
            }
            let payload= Vec::from(&xs[(8+6+6+2)..(8+6+6+2+len)]);
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
            0xAA, 0xAA, 0xAA, 0xAA, 
            0xAA, 0xAA, 0xAA, 0xAB, // preamble
            dst[0], dst[1], dst[2], dst[3], dst[4], dst[5],
            src[0], src[1], src[2], src[3], src[4], src[5],
            et[0], et[1],
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
    (xs[offset] as u64) << 40 | (xs[offset + 1] as u64) << 32 | (xs[offset + 2] as u64) << 24 | (xs[offset + 3] as u64) << 16 | (xs[offset + 4] as u64) << 8 | (xs[offset + 5] as u64)
}

fn split_2bytes(x: u16) -> [u8; 2] {
    [
        (x >> 8) as u8,
        (x & 0xFF) as u8,
    ]
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

trait EtherOperation {
    fn apply(&mut self, ctx: &DeviceContext, port: usize, frame: EthernetFrame) -> Res<Vec<(usize, EthernetFrame)>>;
}

struct Ether {
    op: Box<dyn EtherOperation>,
}
impl DeviceOperation for Ether {
    fn apply(&mut self, ctx: &DeviceContext, port: usize, rbuf: &Vec<u8>) -> Result<Vec<(usize, Vec<u8>)>, PhyslError> {
        match EthernetFrame::decode(rbuf) {
            Ok(frame) => {
                let out_frames = self.op.apply(ctx, port, frame)
                    .map_err(|e| PhyslError::LinklError {e})?;
                let out_bytes = out_frames.iter().map(|(port, frame)| {
                    (port.clone(), EthernetFrame::encode(frame))
                }).collect();
                Ok(out_bytes)
            },
            Err(LinklError::NotEnoughBytes) => Ok(vec![]), // not enough bytes
            Err(e) => Err(
                PhyslError::LinklError{e}
            )
        }

        // // if rbuf has greater than 8 bytes, try read as ethernet frame
        // if rbuf.len() < 8 {
        //     return Ok(vec![]);
        // }
        // let in_frame = EthernetFrame::decode(rbuf)
        //     .map_err(|e| PhyslError::LinklError { msg: format!("failed to decode ether frame: {:?}", e) })?;
        // let out_frames = self.op.apply(ctx, port, in_frame)
        //     .map_err(|e| PhyslError::LinklError { msg: format!("failed to apply ether operation: {:?}", e) })?;
        // let out_bytes = out_frames.iter().map(|(port, frame)| {
        //     (port.clone(), EthernetFrame::encode(frame))
        // }).collect();
        // Ok(out_bytes)
    }
}

struct EtherEcho {}
impl EtherOperation for EtherEcho {
    fn apply(&mut self, _ctx: &DeviceContext, port: usize, frame: EthernetFrame) -> Res<Vec<(usize, EthernetFrame)>> {
        let mut res = Vec::new();
        res.push((port, frame));
        Ok(res)
    }
}

fn build_ether_echo_device(mac: usize, name: String) -> Device {
    let op = Box::new(EtherEcho {});
    let ether = Ether { op };
    let device = Device::new(mac, &name, 1, Box::new(ether));
    device
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

pub fn run_linkl_sample2() -> Result<Vec<u8>, PhyslError> {
    println!("link_sample2 start");

    let mut host = Host::new(0, "host0");
    let echo = build_ether_echo_device(1, "ether0".to_string());
    let xs = vec![
        0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAB, // preamble
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // dst
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02, // src
        0x00, 0x04, // type
        0x01, 0x02, 0x03, 0x04, // payload
    ];
    host.push_to_send(0, &xs)?;

    let mut nw = Network::new();
    nw.add_device(host);
    nw.add_device(echo);
    nw.connect(0, 0, 1, 0)?;

    nw.start(60)?;
    let rbuf = nw.get_receive_buf(0, 0)?;
    println!("res: [");
    for x in rbuf {
        print!("{:02X} ", x)
    }
    println!("]");
    Ok(rbuf.clone())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ether() {
        let rbuf = run_linkl_sample2().unwrap();
        assert_eq!(rbuf.len(), 26);
        assert_eq!(rbuf[24], 0x03);
    }
}