#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IpAddr {
    pub value: u32,
}

impl IpAddr {
    pub fn new(value: u32) -> IpAddr {
        IpAddr { value }
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
        SubnetMask { prefix, value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkPart {
    value: u32,
}

impl NetworkPart {
    pub fn build(ip_addr: IpAddr, subnet_mask: SubnetMask) -> NetworkPart {
        NetworkPart {
            value: ip_addr.value & subnet_mask.value,
        }
    }

    pub fn get_value(&self) -> u32 { self.value }
}