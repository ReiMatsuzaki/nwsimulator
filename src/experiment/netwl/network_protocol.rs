use super::ip::*;
use super::arp::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkProtocol {
    IP(IP),
    ARP(ARP),
}

impl std::fmt::Display for NetworkProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NetworkProtocol::IP(ip) => write!(f, "{}", ip),
            NetworkProtocol::ARP(arp) => write!(f, "{}", arp),
        }
    }
}
