use crate::physl::device::{DeviceContext, Device};

use super::{ethernet::{EthernetOperation, Ethernet}, ethernet_frame::EthernetFrame, linkl_error::Res};

pub struct Schedule {
    pub t: usize,
    pub port: usize,
    pub frame: EthernetFrame,
}

pub struct EthernetHost { schedules: Vec<Schedule> }

impl EthernetHost {
    pub fn build(mac: usize, name: &str, num_ports: usize, schedules: Vec<Schedule>) -> Device {
        let op = Box::new(EthernetHost { schedules });
        let ether = Ethernet { op };
        let device = Device::new(mac, name, num_ports, Box::new(ether));
        device
    }
}

impl EthernetOperation for EthernetHost {
    fn apply(&mut self, ctx: &DeviceContext, _port: usize, frame: EthernetFrame) -> Res<Vec<(usize, EthernetFrame)>> {
        if frame.dst != ctx.mac as u64 {
            Ok(vec![]) // FIXME: remove this frame from receive buffer
        } else {
            Ok(vec![])
        }
    }

    fn update(&mut self, ctx: &DeviceContext) -> Res<Vec<(usize, EthernetFrame)>> {
        let mut res: Vec<(usize, EthernetFrame)> = Vec::new();
        for s in &self.schedules {
            if s.t == ctx.t {
                res.push((s.port, s.frame.clone()));
            }
        }
        Ok(res)        
    }
}
