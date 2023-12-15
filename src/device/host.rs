use super::{Device, DeviceOperation, Res};
use std::collections::VecDeque;

pub struct Host {}

impl Host {
    pub fn new(mac: usize, name: &str) -> Device {
        Device::new(mac, name, 1, Box::new(Host {  }))
    }

}

impl DeviceOperation for Host {
    fn apply(&mut self, _mac: usize, _num_ports: usize, _port: usize, _rbuf: &VecDeque<u8>) -> Res<Vec<(usize, Vec<u8>)>> {
        Ok(Vec::new())
    }
}
