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

    fn push_recv(&mut self, port: Port, x: u8) {
        self.base_mut().recv_buf.push_back((port, x));
    }

    fn pop_send(&mut self) -> Option<(Port, u8)> {
        self.base_mut().send_buf.pop_front()
    }

    fn as_any(&self) -> &dyn Any;

    fn update(&mut self, _ctx: &UpdateContext) -> Res<()>;
}

pub struct BaseDevice {
    mac: Mac,
    name: String,
    num_ports: usize,
    recv_buf: VecDeque<(Port, u8)>,
    send_buf: VecDeque<(Port, u8)>,
}

impl BaseDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseDevice {
        BaseDevice {
            mac,
            name: name.to_string(),
            num_ports,
            recv_buf: VecDeque::new(),
            send_buf: VecDeque::new(),
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

    pub fn recv(&mut self) -> Option<(Port, u8)> {
        self.recv_buf.pop_front()
    }

    pub fn send(&mut self, x: (Port, u8)) {
        self.send_buf.push_back(x)
    }
}

