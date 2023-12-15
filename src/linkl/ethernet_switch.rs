use std::collections::HashMap;

use crate::physl::device::{DeviceContext, Device};

use super::{ethernet::{EthernetOperation, Ethernet}, ethernet_frame::EthernetFrame, linkl_error::Res};

pub struct EthernetSwitch {
    forward_table: HashMap<usize, usize>, // mac -> port
}

impl EthernetSwitch {
    pub fn build(mac: usize, name: String, num_ports: usize) -> Device {
        let op = Box::new(EthernetSwitch { forward_table: HashMap::new() });
        let ether = Ethernet { op };
        let device = Device::new(mac, &name, num_ports, Box::new(ether));
        device
    }
}

impl EthernetOperation for EthernetSwitch {
    fn apply(&mut self, ctx: &DeviceContext, port: usize, frame: EthernetFrame) -> Res<Vec<(usize, EthernetFrame)>> {
        // update forward table
        self.forward_table.insert(frame.src as usize, port);

        let mut res: Vec<(usize, EthernetFrame)> = Vec::new();
        if let Some(dst_port) = self.forward_table.get(&(frame.dst as usize)) {
            res.push((*dst_port, frame));
        } else {
            for p in 0..ctx.num_ports {
                if p != port {
                    res.push((p, frame.clone()));}
            }
        }
        Ok(res)
    }
}

