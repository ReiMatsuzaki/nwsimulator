use crate::experiment::types::{Res, Error};

use super::ip_addr::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IP {
    // FIXME: more fields
    pub src: IpAddr,
    pub dst: IpAddr,
}

impl IP {
    pub fn new(src: IpAddr, dst: IpAddr) -> IP {
        IP { src, dst }
    }

    pub fn decode(xs: &[u8]) -> Res<IP> {
        if xs.len() < 8 {
            return Err(Error::NotEnoughBytes);
        }
        let src = (xs[0] as u32) << 24 | (xs[1] as u32) << 16 | (xs[2] as u32) << 8 | (xs[3] as u32);
        let dst = (xs[4] as u32) << 24 | (xs[5] as u32) << 16 | (xs[6] as u32) << 8 | (xs[7] as u32);
        Ok(IP { src: IpAddr::new(src), dst: IpAddr::new(dst) })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut xs = vec![];
        xs.push((self.src.value >> 24) as u8);
        xs.push((self.src.value >> 16) as u8);
        xs.push((self.src.value >> 8) as u8);
        xs.push(self.src.value as u8);
        xs.push((self.dst.value >> 24) as u8);
        xs.push((self.dst.value >> 16) as u8);
        xs.push((self.dst.value >> 8) as u8);
        xs.push(self.dst.value as u8);
        xs
    }
}

impl std::fmt::Display for IP {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "IP(dst:{}, src:{})", self.dst, self.src)
    }
}