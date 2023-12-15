pub mod linkl_error;
pub mod ethernet;
pub mod ethernet_frame;

use crate::physl::{device::{DeviceContext, Device}, physl_error::PhyslError, host::Host, network::Network};

use self::{ethernet::{EthernetOperation, Ethernet}, ethernet_frame::EthernetFrame, linkl_error::Res};


struct EtherEcho {}
impl EthernetOperation for EtherEcho {
    fn apply(&mut self, _ctx: &DeviceContext, port: usize, frame: EthernetFrame) -> Res<Vec<(usize, EthernetFrame)>> {
        let mut res = Vec::new();
        res.push((port, frame));
        Ok(res)
    }
}

fn build_ether_echo_device(mac: usize, name: String) -> Device {
    let op = Box::new(EtherEcho {});
    let ether = Ethernet { op };
    let device = Device::new(mac, &name, 1, Box::new(ether));
    device
}

pub fn run_linkl_sample2() -> Result<Vec<u8>, PhyslError> {
    println!("link_sample2 start");

    let mut host = Host::new(0, "host0");
    let echo = build_ether_echo_device(1, "ether0".to_string());
    let xs = vec![
        0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAB, // preamble
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // dst
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02, // src
        0x00, 0x04, // type
        0x01, 0x02, 0x03, 0x04, // payload
    ];
    host.push_to_send(0, &xs)?;

    let mut nw = Network::new();
    nw.add_device(host);
    nw.add_device(echo);
    nw.connect(0, 0, 1, 0)?;

    nw.start(60)?;
    let rbuf = nw.get_receive_buf(0, 0)?;
    println!("res: [");
    for x in rbuf {
        print!("{:02X} ", x)
    }
    println!("]");
    Ok(rbuf.clone())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ether() {
        let rbuf = run_linkl_sample2().unwrap();
        assert_eq!(rbuf.len(), 26);
        assert_eq!(rbuf[24], 0x03);
    }
}