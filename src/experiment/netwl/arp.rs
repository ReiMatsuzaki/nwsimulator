use super::super::types::*;
use super::ip_addr::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ARP {
    src_mac: Mac,
    src_ipaddr: IpAddr,
    dst_ipaddr: IpAddr,
}

impl ARP {
    pub fn decode(xs: &[u8]) -> Res<ARP> {
        if xs.len() < 8 {
            return Err(Error::NotEnoughBytes);
        }
        let src = (xs[0] as u32) << 24 | (xs[1] as u32) << 16 | (xs[2] as u32) << 8 | (xs[3] as u32);
        let dst = (xs[4] as u32) << 24 | (xs[5] as u32) << 16 | (xs[6] as u32) << 8 | (xs[7] as u32);
        Ok(ARP { 
            src_mac: Mac::new(0),
            src_ipaddr: IpAddr::new(src), 
            dst_ipaddr: IpAddr::new(dst),
         })
    }

    // pub fn encode(&self) -> Vec<u8> {    
    //     // FIXME
    //     vec![self.src_mac.value as u8,
    //     self.src_ipaddr.value as u8,
    //     self.dst_ipaddr.value as u8
    //     ]
    // }
}