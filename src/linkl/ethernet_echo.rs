use crate::physl::device::{DeviceContext, Device};

use super::{ethernet::{EthernetOperation, Ethernet}, ethernet_frame::EthernetFrame, linkl_error::Res};

struct EtherEcho {}
impl EthernetOperation for EtherEcho {
    fn apply(&mut self, _ctx: &DeviceContext, port: usize, frame: EthernetFrame) -> Res<Vec<(usize, EthernetFrame)>> {
        let mut res: Vec<(usize, EthernetFrame)> = Vec::new();
        let response_frame = EthernetFrame::new(frame.src, frame.dst, frame.ethertype, frame.payload);
        res.push((port, response_frame));
        Ok(res)
    }

    fn update(&mut self, _ctx: &DeviceContext) -> Res<Vec<(usize, EthernetFrame)>> {
        Ok(vec![])
    }
}

// FIXME: move to EtherEcho::build
pub fn build_ether_echo_device(mac: usize, name: String) -> Device {
    let op = Box::new(EtherEcho {});
    let ether = Ethernet { op };
    let device = Device::new(mac, &name, 1, Box::new(ether));
    device
}