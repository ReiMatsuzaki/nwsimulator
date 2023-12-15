use std::collections::VecDeque;

use crate::physl::device::{DeviceContext, Device};

use super::{ethernet::{EthernetOperation, Ethernet}, ethernet_frame::EthernetFrame, linkl_error::Res};

struct EtherEcho {}
impl EthernetOperation for EtherEcho {
    fn apply(&mut self,
        ctx: &DeviceContext, 
        rbufs: &mut Vec<VecDeque<EthernetFrame>>,
        sbufs: &mut Vec<VecDeque<EthernetFrame>>,
       ) -> Res<()> {
        for port in 0..ctx.num_ports {
            let rbuf = &mut rbufs[port];
            while let Some(frame) = rbuf.pop_front() {
                frame.print_msg(Some(&ctx), port, "receive");
                if frame.dst == ctx.mac as u64 {
                    let response = EthernetFrame::new(frame.src, frame.dst, frame.ethertype, frame.payload);
                    let s = format!("send to {}", port);
                    response.print_msg(None, port, s.as_str());

                    sbufs[port].push_back(response);
                } else {
                    frame.print_msg(None, port, "receive");
                }
            }
        }
        Ok(())
    }
}

// FIXME: move to EtherEcho::build
pub fn build_ether_echo_device(mac: usize, name: String) -> Device {
    let op = Box::new(EtherEcho {});
    let ether = Ethernet::new(op, 1);
    let device = Device::new(mac, &name, 1, Box::new(ether));
    device
}