pub mod ethernet_frame;
pub mod ethernet_log;
pub mod ethernet_device;
pub mod ethernet_host;
pub mod ethernet_switch;

pub use ethernet_frame::*;
pub use ethernet_log::*;
pub use ethernet_device::*;
pub use ethernet_host::*;
pub use ethernet_switch::*;

use super::types::{Port, Mac, Res};
use super::physl::Network;


pub fn run_sample() -> Res<EthernetLog> {
    println!("run experimental linkl sample");
    crate::output::set_level(crate::output::Level::Frame);
    let mac0 = Mac::new(23);
    let mac1 = Mac::new(24);
    let mac2 = Mac::new(25);

    let mut host_a = EthernetHost::build_echo(mac0, "host_a");
    let host_b = EthernetHost::build_echo(mac1, "host_b");
    let brdige = EthernetSwitch::build_bridge(mac2, "bridge");

    let frame = EthernetFrame::new(mac1, mac0, 3, vec![11, 12, 13]);
    host_a.add_schedule(0, frame.clone());

    let mut nw = Network::new(
        vec![host_a, host_b, brdige],
        vec![]
    );
    nw.connect_both(mac0, Port::new(0), mac2, Port::new(0)).unwrap();
    nw.connect_both(mac1, Port::new(0), mac2, Port::new(1)).unwrap();
    nw.run(60).unwrap();

    let d = nw.get_device(mac1).unwrap();
    let d = d.as_any().downcast_ref::<EthernetHost>().unwrap();
    println!("{}", d.get_rlog().len());
    println!("{:?}", d.get_rlog()[0]);
    Ok(d.get_rlog()[0].clone())
}

pub fn run_sample_3host() -> Res<EthernetLog> {
    println!("run experimental linkl 3host sample");
    crate::output::set_level(crate::output::Level::Frame);
    let mac0 = Mac::new(21);
    let mac1 = Mac::new(22);
    let mac2 = Mac::new(23);
    let mac3 = Mac::new(24);
    let mac_s = Mac::new(30);

    let mut host_0 = EthernetHost::build_consumer(mac0, "host_a");
    let host_1 = EthernetHost::build_echo(mac1, "host_b");
    let host_2 = EthernetHost::build_echo(mac2, "host_c");
    let host_3 = EthernetHost::build_echo(mac3, "host_d");
    let switch = EthernetSwitch::build_switch(mac_s, "switch", 4);

    let frame = EthernetFrame::new(mac1, mac0, 3, vec![11, 12, 13]);
    host_0.add_schedule(0, frame.clone());

    let mut nw = Network::new(
        vec![host_0, host_1, host_2, host_3, switch],
        vec![]
    );
    nw.connect_both(mac_s, Port::new(0), mac0, Port::new(0))?;
    nw.connect_both(mac_s, Port::new(1), mac1, Port::new(0))?;
    nw.connect_both(mac_s, Port::new(2), mac2, Port::new(0))?;
    nw.connect_both(mac_s, Port::new(3), mac3, Port::new(0))?;
    nw.run(200).unwrap();

    let d = nw.get_device(mac0)?;
    let d = d.as_any().downcast_ref::<EthernetHost>().unwrap();
    println!("{}", d.get_rlog().len());
    let log = &d.get_rlog()[0];
    println!("t={}, frame={}", log.t, log.frame);
    // Ok(d.get_slog()[0].clone())
    Ok(log.clone())
}

#[cfg(test)]
mod tests {
    use super::super::netwl::{ip::IP, ip_addr::IpAddr};

    use super::*;

    #[test]
    fn test_ethernet_frame() {
        let ip = IP::new_byte(IpAddr::new(180), IpAddr::new(240), vec![1, 2, 3]);
        let ip_byte: Vec<u8> = ip.encode();
        // println!("ip: {:?}", ip_byte);
        let frame = EthernetFrame::new(Mac::new(1), Mac::new(2), 0x0800, ip_byte);
        let xs = EthernetFrame::encode(&frame);
        let frame2 = EthernetFrame::decode(&xs).unwrap();
        println!("frame: {:?}", frame);
        assert_eq!(frame, frame2);
    }

    #[test]
    fn test_2host_1bridge() {
        let log = run_sample().unwrap();
        let mac0 = Mac::new(23);
        let mac1 = Mac::new(24);
        let frame = EthernetFrame::new(mac1, mac0, 3, vec![11, 12, 13]);
        assert_eq!(frame, log.frame);
    }

    #[test]
    fn test_4host_1switch() {
        let log = run_sample_3host().unwrap();
        let mac0 = Mac::new(21);
        let mac1 = Mac::new(22);
        let frame = EthernetFrame::new(mac0, mac1, 3, vec![11, 12, 13]);
        assert_eq!(frame, log.frame);
    }
}