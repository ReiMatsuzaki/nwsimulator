use super::super::utils::{read_6bytes, read_2bytes, split_6bytes, split_2bytes};
use super::super::types::{Mac, Res, Error};

#[derive(Clone, Debug, PartialEq)]
pub struct EthernetFrame {
    pub dst: Mac,       // 6 bytes
    pub src: Mac,       // 6 bytes
    pub ethertype: u16, // 2 bytes
    pub payload: Vec<u8>,
}

pub const MAC_BROADCAST: Mac = Mac { value: 0xFFFFFFFFFFFF };

impl EthernetFrame {
    pub fn new(dst: Mac, src: Mac, ethertype: u16, payload: Vec<u8>) -> EthernetFrame {
        EthernetFrame {
            dst,
            src,
            ethertype,
            payload,
        }
    }

    // pub fn new_bloadcast(src: Mac, ethertype: u16, payload: Vec<u8>) -> EthernetFrame {
    //     EthernetFrame {
    //         dst: MAC_BLOADCAST,
    //         src,
    //         ethertype,
    //         payload,
    //     }
    // }

    pub fn is_bloadcast(&self) -> bool {
        self.dst == MAC_BROADCAST
    }

    pub fn decode(xs: &Vec<u8>) -> Res<EthernetFrame> {
        if xs.len() < 8 + 6 + 6 + 2 {
            return Err(Error::NotEnoughBytes);
        }
        for i in 0..7 {
            if xs[i] != 0xAA {
                // 10101010 = 0xAA
                return Err(Error::InvalidBytes {
                    msg: "bad preamble".to_string(),
                });
            }
        }
        if xs[7] != 0xAB {
            // 10101011 = 0xAB
            return Err(Error::InvalidBytes {
                msg: "bad preamble".to_string(),
            });
        }
        let dst = Mac::new(read_6bytes(xs, 8));
        let src = Mac::new(read_6bytes(xs, 8 + 6));
        let ty = read_2bytes(xs, 8 + 6 + 6);
        let len = match ty {
            0x0800 => { 
                // IPv4
                let i = 8+6+6+2;
                if xs.len() < i + 2 + 2 {
                    return Err(Error::NotEnoughBytes);
                }
                read_2bytes(xs, i+2) as usize
            }
            0x0806 => 24, // ARP
            ty if ty <= 0x05DC => ty as usize,
            _ => {
                return Err(Error::InvalidBytes {
                    msg: format!("unsupported ethernet type: {}", ty),
                });
            }
        };
        if xs.len() < 8 + 6 + 6 + 2 + len {
            return Err(Error::NotEnoughBytes);
        }
        let payload = Vec::from(&xs[8+6+6+2..8+6+6+2+len]);
        Ok(EthernetFrame {
            dst,
            src,
            ethertype: ty,
            payload,
        })
    }

    pub fn encode(frame: &EthernetFrame) -> Vec<u8> {
        let et = split_2bytes(frame.ethertype);
        let dst = split_6bytes(frame.dst.value);
        let src = split_6bytes(frame.src.value);
        let mut xs = vec![
            0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAB, // preamble
            dst[0], dst[1], dst[2], dst[3], dst[4], dst[5], src[0], src[1], src[2], src[3], src[4],
            src[5], et[0], et[1],
        ];
        xs.append(&mut frame.payload.clone());
        xs
    }
}

impl std::fmt::Display for EthernetFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut payload = "[".to_string();
        for x in &self.payload {
            payload = format!("{} {}", payload, x);
        }
        payload = format!("{}]", payload);
        write!(f, "EthernetFrame(dst:{}, src:{}, typ:{}, payload:{})", 
               self.dst.value, self.src.value, self.ethertype, payload)
    }
}

