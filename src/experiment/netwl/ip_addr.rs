#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IpAddr {
    pub value: u32,
}

impl IpAddr {
    pub fn new(value: u32) -> IpAddr {
        IpAddr { value }
    }
}

impl std::fmt::Display for IpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut value = self.value;
        let mut xs = vec![];
        for _ in 0..4 {
            xs.push((value & 0xff) as u8);
            value = value >> 8;
        }
        // xs.reverse();
        write!(f, "{}.{}.{}.{}", xs[3], xs[2], xs[1], xs[0])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubnetMask {
    pub prefix: u8,
    pub value: u32,
}

impl SubnetMask {
    pub fn new(prefix: u8) -> SubnetMask {
        let mut value = 0;
        for _ in 0..prefix {
            value = value << 1;
            value += 1;
        }
        for _ in 0..(32-prefix) {
            value  = value << 1;
        }
        SubnetMask { prefix, value }
    }
}

impl std::fmt::Display for SubnetMask {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ip_addr = IpAddr::new(self.value);
        write!(f, "{}", ip_addr)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkPart {
    value: u32,
    prefix: u8,
}

impl NetworkPart {
    pub fn new(ip_addr: IpAddr, subnet_mask: SubnetMask) -> NetworkPart {
        NetworkPart {
            value: ip_addr.value & subnet_mask.value,
            prefix: subnet_mask.prefix,
        }
    }
    // pub fn get_value(&self) -> u32 { self.value }
}

impl std::fmt::Display for NetworkPart {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", IpAddr::new(self.value), self.prefix)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_addr() {
        let ip_addr = IpAddr::new(0b11000000_10101000_00000001_00000010);
        let subnet_mask = SubnetMask::new(24);
        let network_part = NetworkPart::new(ip_addr, subnet_mask);
        println!("----------------");
        println!("{}", ip_addr);
        println!("{}", subnet_mask);
        println!("{}", network_part);
        println!("----------------");
        assert_eq!(subnet_mask.prefix, 24);
        assert_eq!(subnet_mask.value, 0b11111111_11111111_11111111_00000000);        
        assert_eq!(network_part.value, 0b11000000_10101000_00000001_00000000);
    }
}