use std::{collections::VecDeque, any::Any};
use super::super::types::*;

pub trait Device {
    fn base(&self) -> &BaseDevice;

    fn base_mut(&mut self) -> &mut BaseDevice;

    fn get_mac(&self) -> Mac {
        self.base().mac
    }

    fn get_name(&self) -> &str {
        &self.base().name
    }

    fn get_num_ports(&self) -> usize {
        self.base().num_ports
    }

    fn push_rbuf(&mut self, port: Port, x: u8) {
        self.base_mut().rbuf.push_back((port, x));
    }

    fn pop_sbuf(&mut self) -> Option<(Port, u8)> {
        self.base_mut().sbuf.pop_front()
    }

    fn as_any(&self) -> &dyn Any;

    fn update(&mut self, _ctx: &UpdateContext) -> Res<()>;
}

pub struct BaseDevice {
    mac: Mac,
    name: String,
    num_ports: usize,
    rbuf: VecDeque<(Port, u8)>,
    sbuf: VecDeque<(Port, u8)>,
}

impl BaseDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseDevice {
        BaseDevice {
            mac,
            name: name.to_string(),
            num_ports,
            rbuf: VecDeque::new(),
            sbuf: VecDeque::new(),
        }
    }

    pub fn get_mac(&self) -> Mac {
        self.mac
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_num_ports(&self) -> usize {
        self.num_ports
    }

    pub fn pop_rbuf(&mut self) -> Option<(Port, u8)> {
        self.rbuf.pop_front()
    }

    pub fn push_sbuf(&mut self, x: (Port, u8)) {
        self.sbuf.push_back(x)
    }
}

