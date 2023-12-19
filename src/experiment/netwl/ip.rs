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
    pub payload: IpPayload,
}

impl IP {
    // FIXME: rename
    pub fn new_byte(src: IpAddr, dst: IpAddr, payload: Vec<u8>) -> IP {
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
            payload: IpPayload::Bytes(payload),
         }
    }

    // FIXME: rename
    pub fn new(src: IpAddr, dst: IpAddr, protocol: u8, payload: IpPayload) -> IP {
        IP { 
            version_ihl: 0x45,
            tos: 0,
            total_length: 20 + payload.len() as u16,
            id: 0,
            flags_fragment_offset: 0,
            ttl: 64,
            protocol,
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
        let mut i = 0;
        let protocol = xs[i];
        i += 1;
        // FIXME: deplicated code
        let src = (xs[i] as u32) << 24 | (xs[i+1] as u32) << 16 | (xs[i+2] as u32) << 8 | (xs[i+3] as u32);
        i += 4;
        let dst = (xs[i] as u32) << 24 | (xs[i+1] as u32) << 16 | (xs[i+2] as u32) << 8 | (xs[i+3] as u32);
        i += 4;
        let payload= if protocol == 1 {
            IpPayload::ICMP { ty: xs[i], code: xs[i+1] }
        } else {
            IpPayload::Bytes(xs[9..].to_vec())
        };
        Ok(IP::new(IpAddr::new(src), IpAddr::new(dst), protocol, payload))
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut xs = vec![];
        xs.push(self.protocol);
        xs.push((self.src.value >> 24) as u8);
        xs.push((self.src.value >> 16) as u8);
        xs.push((self.src.value >> 8) as u8);
        xs.push(self.src.value as u8);
        xs.push((self.dst.value >> 24) as u8);
        xs.push((self.dst.value >> 16) as u8);
        xs.push((self.dst.value >> 8) as u8);
        xs.push(self.dst.value as u8);
        match &self.payload {
            IpPayload::Bytes(xs2) => {
                for x in xs2 {
                    xs.push(*x);
                }
            },
            IpPayload::ICMP { ty, code } => {
                xs.push(*ty);
                xs.push(*code);
            }
        }
        xs
    }
}

impl std::fmt::Display for IP {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // FIXME: duplicated code
        let payload = match &self.payload {
            IpPayload::Bytes(xs) => {
                let mut payload = "[".to_string();
                for x in xs {
                    payload = format!("{} {:0>2X}", payload, x);
                }
                payload = format!("{}]", payload);
                payload
            },
            IpPayload::ICMP { ty, code } => {
                format!("ICMP[type:{:0>2X}, code:{:0>2X}]", ty, code)
            }
        };
        
        write!(f, "IP(dst:{}, src:{}, payload:{})", self.dst, self.src, payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpPayload {
    Bytes(Vec<u8>),
    ICMP { ty: u8, code: u8 },
}

impl IpPayload {
    pub fn len(&self) -> usize {
        match self {
            IpPayload::Bytes(xs) => xs.len(),
            IpPayload::ICMP { ty: _, code: _ } => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip() {
        let ips = vec![
            IP::new_byte(IpAddr::new(0x0a000001), IpAddr::new(0x0a000002), vec![0x01, 0x02, 0x03]),
            IP::new(IpAddr::new(0x0a000001), IpAddr::new(0x0a000002), 1, IpPayload::ICMP{ty:2, code:3})
        ];
        for ip in ips {
            let xs = ip.encode();
            let ip2 = IP::decode(&xs).unwrap();
            assert_eq!(ip, ip2);
        }
    }
}