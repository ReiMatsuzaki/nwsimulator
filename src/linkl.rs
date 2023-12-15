use crate::{physl::{host::Host, network::Network, physl_error::PhyslError}, linkl::ethernet_echo::build_ether_echo_device};

use self::{ethernet_switch::EthernetSwitch, ethernet_frame::EthernetFrame};

pub mod linkl_error;
pub mod ethernet;
pub mod ethernet_frame;
pub mod ethernet_echo;
pub mod ethernet_switch;

pub fn run_linkl_sample2() -> Result<Vec<u8>, PhyslError> {
    println!("link_sample2 start");

    let mut host = Host::new(0, "host0");
    let echo = build_ether_echo_device(1, "ether0".to_string());
    let xs: Vec<u8> = vec![
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

pub fn run_sample_ethernet_switch() -> Result<Network, PhyslError> {
    println!("link_sample_ethernet_switch start");
    let mac_a = 0;
    let mac_b = 1;
    let mac_c = 2;
    let mac_s = 3;
    let mut host_a = Host::new(mac_a, "hostA");
    // let host_b = Host::new(1, "hostB");
    let host_b = build_ether_echo_device(mac_b, "ether0".to_string());
    let host_c = Host::new(mac_c, "HostC");
    let switch = EthernetSwitch::build(mac_s, "switch".to_string(), 3);

    let frame = EthernetFrame::new(
        mac_b as u64, mac_a as u64, 3, vec![11, 22, 33]);
    let xs: Vec<u8> = EthernetFrame::encode(&frame);
    // xs.append(&mut xs.clone());
    host_a.push_to_send(0, &xs)?;

    let mut nw = Network::new();
    let mac_a = mac_a as usize;
    let mac_b = mac_b as usize;
    nw.add_device(host_a);
    nw.add_device(host_b);
    nw.add_device(host_c);
    nw.add_device(switch);
    nw.connect(mac_s, 0, mac_a, 0)?;
    nw.connect(mac_s, 1, mac_b, 0)?;
    nw.connect(mac_s, 2, mac_c, 0)?;    

    nw.start(100)?;    

    for mac in [mac_a, mac_b, mac_c] {
        let rbuf = nw.get_receive_buf(mac, 0)?;
        // let frame = EthernetFrame::decode(&rbuf).unwrap();
        // println!("Host(mac={}): {:?}", mac, frame);
        print!("Host(mac={}): [", mac,);
        for x in rbuf {
            print!("{:02X} ", x)
        }
        println!("]");
    }
    Ok(nw)
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

    #[test]
    fn test_ethernet_switch() {
        let nw = run_sample_ethernet_switch().unwrap();
        let rbuf = nw.get_receive_buf(0, 0).unwrap();
        assert_eq!(rbuf.len(), 25);
        let rbuf = nw.get_receive_buf(1, 0).unwrap();
        assert_eq!(rbuf.len(), 0);
        let rbuf = nw.get_receive_buf(2, 0).unwrap();
        assert_eq!(rbuf.len(), 25);
    }
}