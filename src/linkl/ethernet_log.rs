use super::EthernetFrame;


#[derive(Debug, Clone)]
pub struct EthernetLog {
    pub t: usize,
    pub frame: EthernetFrame,
}