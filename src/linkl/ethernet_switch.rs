use std::collections::{HashMap, VecDeque};

use crate::physl::device::{DeviceContext, Device};

use super::{ethernet::{EthernetOperation, Ethernet}, ethernet_frame::EthernetFrame, linkl_error::Res};

pub struct EthernetSwitch {
    forward_table: HashMap<usize, usize>, // mac -> port
}

impl EthernetSwitch {
    pub fn build(mac: usize, name: String, num_ports: usize) -> Device {
        let op = Box::new(EthernetSwitch { forward_table: HashMap::new() });
        let ether = Ethernet::new(op, num_ports);
        let device = Device::new(mac, &name, num_ports, Box::new(ether));
        device
    }
}

impl EthernetOperation for EthernetSwitch {
    fn apply(&mut self,
        ctx: &DeviceContext, 
        rbufs: &mut Vec<VecDeque<EthernetFrame>>,
        sbufs: &mut Vec<VecDeque<EthernetFrame>>,
       ) -> Res<()> {
        for (port, rbuf) in rbufs.iter_mut().enumerate() {
            while let Some(frame) = rbuf.pop_front() {
                let n = format!("{}:{}", ctx.name, port);
                println!("t={:<3}  {:<15}  receive:  {:?}", ctx.t, n, frame);

                // update forward table
                self.forward_table.insert(frame.src as usize, port);

                if let Some(dst_port) = self.forward_table.get(&(frame.dst as usize)) {
                    // use forward table
                    sbufs[*dst_port].push_back(frame.clone());
                    println!("{}  send to {}:  {:?}", " ".repeat(22), *dst_port, frame);
                } else {
                    for dst_port in 0..ctx.num_ports {
                        if dst_port != port {
                            sbufs[dst_port].push_back(frame.clone());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

