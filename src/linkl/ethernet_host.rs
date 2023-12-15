use std::collections::VecDeque;

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
        let ether = Ethernet::new(op, num_ports);
        let device = Device::new(mac, name, num_ports, Box::new(ether));
        device
    }
}

impl EthernetOperation for EthernetHost {
    fn apply(&mut self,
        ctx: &DeviceContext, 
        rbufs: &mut Vec<VecDeque<EthernetFrame>>,
        sbufs: &mut Vec<VecDeque<EthernetFrame>>,
       ) -> Res<()> {
        for port in 0..ctx.num_ports {
            let f = &mut rbufs[port];
            while let Some(frame) = f.pop_front() {
                if frame.dst == ctx.mac as u64 {
                    frame.print_msg(Some(&ctx), port, "receive(valid)");
                } else {
                    frame.print_msg(Some(&ctx), port, "receive(invalid)");
                }
            }
        }

        for s in &self.schedules {
            if s.t == ctx.t {
                sbufs[s.port].push_back(s.frame.clone());
            }
        }

        Ok(())
    }
}
