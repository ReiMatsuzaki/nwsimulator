use crate::experiment::types::{Res, Error};

use super::ip_addr::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IP {
    // FIXME: more fields
    version_ihl: u8,
    tos: u8,
    total_length: u16,
    id: u16,
    flags_fragment_offset: u16,
    ttl: u8,
    protocol: u8,
    checksum: u16,
    pub src: IpAddr,
    pub dst: IpAddr,
    pub payload: Vec<u8>,
}

impl IP {
    pub fn new(src: IpAddr, dst: IpAddr, payload: Vec<u8>) -> IP {
        IP { 
            version_ihl: 0x45,
            tos: 0,
            total_length: 20 + payload.len() as u16,
            id: 0,
            flags_fragment_offset: 0,
            ttl: 64,
            protocol: 0,
            checksum: 0,
            src, 
            dst,
            payload,
         }
    }

    pub fn decode(xs: &[u8]) -> Res<IP> {
        if xs.len() < 8 {
            return Err(Error::NotEnoughBytes);
        }
        let src = (xs[0] as u32) << 24 | (xs[1] as u32) << 16 | (xs[2] as u32) << 8 | (xs[3] as u32);
        let dst = (xs[4] as u32) << 24 | (xs[5] as u32) << 16 | (xs[6] as u32) << 8 | (xs[7] as u32);
        let payload = xs[8..].to_vec();
        Ok(IP::new(IpAddr::new(src), IpAddr::new(dst), payload))
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
        for x in &self.payload {
            xs.push(*x);
        }
        xs
    }
}

impl std::fmt::Display for IP {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // FIXME: duplicated code
        let mut payload = "[".to_string();
        for x in &self.payload {
            payload = format!("{} {}", payload, x);
        }
        payload = format!("{}]", payload);
        write!(f, "IP(dst:{}, src:{}, payload:{})", self.dst, self.src, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip() {
        let ip = IP::new(IpAddr::new(0x0a000001), IpAddr::new(0x0a000002), vec![0x01, 0x02, 0x03]);
        let xs = ip.encode();
        let ip2 = IP::decode(&xs).unwrap();
        assert_eq!(ip, ip2);
    }
}