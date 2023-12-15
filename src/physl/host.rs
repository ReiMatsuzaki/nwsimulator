use std::collections::VecDeque;

use super::{device::{DeviceOperation, Device, DeviceContext}, physl_error::Res};

pub struct Host {}

impl Host {
    pub fn new(mac: usize, name: &str) -> Device {
        Device::new(mac, name, 1, Box::new(Host {  }))
    }

}

impl DeviceOperation for Host {
    fn apply(&mut self, _ctx: &DeviceContext, _port: usize, _rbuf: &VecDeque<u8>) -> Res<Vec<(usize, Vec<u8>)>> {
        Ok(Vec::new())
    }
}
