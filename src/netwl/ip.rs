use crate::{types::{Res, Error}, utils::read_2bytes};

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

    pub fn new_icmp(src: IpAddr, dst: IpAddr, ty: u8, code: u8) -> IP {
        IP::new(src, dst, IpPayload::ICMP{ty, code})
    }

    pub fn new(src: IpAddr, dst: IpAddr, payload: IpPayload) -> IP {
        IP { 
            version_ihl: 0x45,
            tos: 0,
            total_length: 20 + payload.len() as u16,
            id: 0,
            flags_fragment_offset: 0,
            ttl: 64,
            protocol: payload.protocol(),
            checksum: 0,
            src, 
            dst,
            payload,
         }        
    }

    pub fn decode(xs: &[u8]) -> Res<IP> {
        let xs = Vec::from(xs);
        if xs.len() < 8 {
            return Err(Error::NotEnoughBytes);
        }
        let mut i = 0;
        let version_ihl = xs[i]; i+= 1;
        let tos = xs[i]; i+= 1;
        let total_length = read_2bytes(&xs, i); i+= 2;
        let id = read_2bytes(&xs, i); i+=2;
        let flags_fragment_offset = read_2bytes(&xs, i); i+=2;
        let ttl = xs[i]; i+= 1;
        let protocol = xs[i]; i+= 1;
        let checksum = read_2bytes(&xs, i); i+=2;
        // FIXME: deplicated code
        let src = IpAddr::new((xs[i] as u32) << 24 | (xs[i+1] as u32) << 16 | (xs[i+2] as u32) << 8 | (xs[i+3] as u32));
        i += 4;
        let dst = IpAddr::new((xs[i] as u32) << 24 | (xs[i+1] as u32) << 16 | (xs[i+2] as u32) << 8 | (xs[i+3] as u32));
        i += 4;
        let payload= if protocol == 1 {
            IpPayload::ICMP { ty: xs[i], code: xs[i+1] }
        } else {
            IpPayload::Bytes(xs[i..].to_vec())
        };
        Ok(IP { 
            version_ihl,
            tos,
            total_length,
            id,
            flags_fragment_offset,
            ttl,
            protocol,
            checksum,
            src,
            dst,
            payload,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut xs = vec![];
        xs.push(self.version_ihl);
        xs.push(self.tos);
        xs.append(&mut self.total_length.to_be_bytes().to_vec());
        xs.append(&mut self.id.to_be_bytes().to_vec());
        xs.append(&mut self.flags_fragment_offset.to_be_bytes().to_vec());
        xs.push(self.ttl);
        xs.push(self.protocol);
        xs.append(&mut self.checksum.to_be_bytes().to_vec());
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

    fn protocol(&self) -> u8 {
        match self {
            IpPayload::Bytes(_) => 0, // it is true
            IpPayload::ICMP { ty: _, code: _ } => 1,
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
            IP::new(IpAddr::new(0x0a000001), IpAddr::new(0x0a000002), IpPayload::ICMP{ty:2, code:3})
        ];
        for ip in ips {
            let xs = ip.encode();
            let ip2 = IP::decode(&xs).unwrap();
            assert_eq!(ip, ip2);
        }
    }
}