use crate::experiment::utils::{read_2bytes, read_4bytes};

use super::super::types::*;
use super::ip_addr::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ARP {
    hardware_type: u16,
    protocol_type: u16,
    hardware_size: u8,
    protocol_size: u8,
    opcode: u16,
    sender_mac: Mac, // u32
    sender_ipaddr: IpAddr,
    target_mac: Mac,
    target_ipaddr: IpAddr,
}

impl ARP {
    pub fn decode(xs: &[u8]) -> Res<ARP> {
        let xs = Vec::from(xs);
        if xs.len() < 24 {
            return Err(Error::NotEnoughBytes);
        }
        let mut i = 0;
        let hardware_type = read_2bytes(&xs, i);
        i += 2;
        let protocol_type = read_2bytes(&xs, i);
        i += 2;
        let hardware_size = xs[i];
        i += 1;
        let protocol_size = xs[i];
        i += 1;
        let opcode = read_2bytes(&xs, i);
        i += 2;
        let sender_mac = Mac::new(read_4bytes(&xs, i) as u64);
        i += 4;
        let sender_ipaddr = IpAddr::new(read_4bytes(&xs, i));
        i += 4;
        let target_mac = Mac::new(read_4bytes(&xs, i) as u64);
        i += 4;
        let target_ipaddr = IpAddr::new(read_4bytes(&xs, i));

        Ok(ARP { 
            hardware_type,
            protocol_type,
            hardware_size,
            protocol_size,
            opcode,
            sender_mac,
            sender_ipaddr,
            target_mac,
            target_ipaddr,
         })
    }

    pub fn encode(&self) -> Vec<u8> {    
        let mut xs = vec![];
        xs.append(&mut self.hardware_type.to_be_bytes().to_vec());
        xs.append(&mut self.protocol_type.to_be_bytes().to_vec());
        xs.push(self.hardware_size);
        xs.push(self.protocol_size);
        xs.append(&mut self.opcode.to_be_bytes().to_vec());
        let u = self.sender_mac.value as u32;
        xs.append(&mut u.to_be_bytes().to_vec());
        let u = self.sender_ipaddr.value as u32;
        xs.append(&mut u.to_be_bytes().to_vec());
        let u = self.target_mac.value as u32;
        xs.append(&mut u.to_be_bytes().to_vec());
        let u = self.target_ipaddr.value as u32;
        xs.append(&mut u.to_be_bytes().to_vec());
        xs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arp() {
        let arp = ARP {
            hardware_type: 1,
            protocol_type: 0x0800,
            hardware_size: 6,
            protocol_size: 4,
            opcode: 1,
            sender_mac: Mac::new(0x01020304),
            sender_ipaddr: IpAddr::new(0x0a000001),
            target_mac: Mac::new(0x03040506),
            target_ipaddr: IpAddr::new(0x0a000002),
        };
        let xs = arp.encode();
        let arp2 = ARP::decode(&xs).unwrap();
        assert_eq!(arp, arp2);
    }
}