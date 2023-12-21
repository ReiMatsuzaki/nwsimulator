pub mod ip_addr;
pub mod ip;
pub mod arp;
pub mod network_protocol;
pub mod ip_device;
pub mod ip_host;

use super::linkl::EthernetSwitch;

use super::types::{Port, Mac, Res};
use super::physl::Network;

use ip_addr::*;
use ip::*;
use arp::*;
use network_protocol::*;
use ip_device::*;
use ip_host::*;

#[derive(Debug)]
pub struct NetworkLog {
    t: usize,
    p: NetworkProtocol,
}

pub fn run_host_host() -> Res<()> {
    crate::output::set_level(crate::output::Level::Frame);
    let subnet_mask = SubnetMask::new(24);
    let addr0 = IpAddr::new(0x0a00_0001);
    let addr1 = IpAddr::new(0x0a00_0002);
    let ip0 = IP::new_byte(addr0, addr1, vec![0x01, 0x02]);
    let mac0 = Mac::new(761);
    let mac1 = Mac::new(762);
    let port0 = Port::new(0);
    let port1 = Port::new(0);
    let mut host0 = IpHost::build_echo(mac0, "host1", addr0, subnet_mask);
    let mut host1 = IpHost::build_echo(mac1, "host2", addr1, subnet_mask);
    host0.add_schedule(0, NetworkProtocol::IP(ip0));
    host0.add_arp_entry(addr1, mac1)?;
    host1.add_arp_entry(addr0, mac0)?;

    let mut nw = Network::new(
        vec![host0, host1],
        vec![]
    );
    nw.connect_both(mac0, port0, mac1, port1).unwrap();
    nw.run(100).unwrap();
    let d = nw.get_device(mac0).unwrap();
    let d = d.as_any().downcast_ref::<IpHost>().unwrap();
    // println!("{}", d.get_name());    
    // let log = &d.get_rlog()[0];
    // println!("received log: {:?}, {:?}", log.t, log.p);
    println!("{:?}", d.get_rlog());
    assert_eq!(1, d.get_rlog().len());
    let ip = match d.get_rlog()[0].p.clone() {
        NetworkProtocol::IP(p) => p,
        _ => panic!("")
    };
    assert_eq!(d.get_ip_addr(Port::new(0)), Some(ip.dst));
    Ok(())
}

pub fn run_2host_1router() -> Res<()> {
    crate::output::set_level(crate::output::Level::Frame);
    let subnet_mask = SubnetMask::new(24);
    let addr_a = IpAddr::new(0x0a00_0001);
    let addr_b = IpAddr::new(0x0a00_0002);
    let addr_r = IpAddr::new(0x0a00_0003);

    let mac_a = Mac::new(761);
    let mac_b = Mac::new(762);
    let mac_s = Mac::new(763);
    let mac_r = Mac::new(764);

    let mut host_a = IpHost::build_echo(mac_a, "hostA", addr_a, subnet_mask);
    let ip0 = IP::new_byte(addr_a, addr_b, vec![0x01, 0x02]);
    host_a.add_schedule(0, NetworkProtocol::IP(ip0));
    host_a.add_arp_entry(addr_b, mac_b)?;

    let mut host_b = IpHost::build_echo(mac_b, "hostB", addr_b, subnet_mask);
    host_b.add_arp_entry(addr_a, mac_a)?;

    let switch = EthernetSwitch::build_switch(mac_s, "switch", 3);
    let router = BaseIpDevice::new_router(mac_r, "router", vec![addr_r], subnet_mask);

    let mut nw = Network::new(
        vec![host_a, host_b, switch, router],
        vec![]
    );
    nw.connect_both(mac_s, Port::new(0), mac_a, Port::new(0))?;
    nw.connect_both(mac_s, Port::new(1), mac_b, Port::new(0))?;
    nw.connect_both(mac_s, Port::new(2), mac_r, Port::new(0))?;

    nw.run(200).unwrap();
    let d = nw.get_device(mac_a).unwrap();
    let d = d.as_any().downcast_ref::<IpHost>().unwrap();

    println!("{:?}", d.get_rlog());
    assert_eq!(1, d.get_rlog().len());
    let log = &d.get_rlog()[0];
    println!("t={}, p={}", log.t, log.p);
    let ip = match log.p.clone() {
        NetworkProtocol::IP(p) => p,
        _ => panic!("")
    };
    assert_eq!(d.get_ip_addr(Port::new(0)), Some(ip.dst));
    // println!("{}", d.get_name());    

    Ok(())    
}

pub fn run_2router() -> Res<()> {
    crate::output::set_level(crate::output::Level::Frame);
    let subnet_mask = SubnetMask::new(24);

    let addr_1a = IpAddr::new(0x0a01_0001);
    let addr_1b = IpAddr::new(0x0a01_0002);
    let addr_1r = IpAddr::new(0x0a01_0003);
    let addr_2r = IpAddr::new(0x0a02_0001);
    let addr_2s = IpAddr::new(0x0a02_0002);
    let addr_3c = IpAddr::new(0x0a03_0001);
    let addr_3d = IpAddr::new(0x0a03_0002);
    let addr_3s = IpAddr::new(0x0a03_0003);

    let mac_a = Mac::new(761);
    let mac_b = Mac::new(762);
    let mac_c = Mac::new(763);
    let mac_d = Mac::new(764);
    let mac_r = Mac::new(765);
    let mac_s = Mac::new(766);
    let mac_1 = Mac::new(767);
    let mac_3 = Mac::new(768);

    let mut host_a = IpHost::build_echo(mac_a, "host1a", addr_1a, subnet_mask);
    let host_b = IpHost::build_echo(mac_b, "host1b", addr_1b, subnet_mask);
    let host_c = IpHost::build_echo(mac_c, "host3c", addr_3c, subnet_mask);
    let mut host_d = IpHost::build_echo(mac_d, "host3d", addr_3d, subnet_mask);
    let mut router_r = BaseIpDevice::new_router(mac_r, "routeR", vec![addr_1r, addr_2r], subnet_mask);
    let mut router_s = BaseIpDevice::new_router(mac_s, "routeS", vec![addr_3s, addr_2s], subnet_mask);
    let switch_1 = EthernetSwitch::build_switch(mac_1, "switch1", 3);
    let switch_3 = EthernetSwitch::build_switch(mac_3, "switch3", 3);

    let ip0 = IP::new_byte(addr_1a, addr_3d, vec![0x01, 0x02]);
    host_a.add_schedule(0, NetworkProtocol::IP(ip0));

    host_a.add_arp_entry(addr_1r, mac_r)?;
    host_d.add_arp_entry(addr_3s, mac_s)?;
    router_r.add_arp_entry(addr_1a, mac_a)?;
    router_r.add_arp_entry(addr_2s, mac_s)?;
    router_s.add_arp_entry(addr_3d, mac_d)?;
    router_s.add_arp_entry(addr_2r, mac_r)?;

    let nw_part = addr_3s.nw(subnet_mask);
    host_a.add_route_entry(nw_part, addr_1r)?;
    router_r.add_route_entry(nw_part, addr_2s)?;
    router_s.add_route_entry(nw_part, addr_3d)?;

    let nw1_part = addr_1r.nw(subnet_mask);
    host_d.add_route_entry(nw1_part, addr_3s)?;
    router_s.add_route_entry(nw1_part, addr_2r)?;

    let mut nw = Network::new(
        vec![host_a, host_b, host_c, host_d, router_r, router_s, switch_1, switch_3],
        vec![]
    );
    nw.connect_both(mac_1, Port::new(0), mac_a, Port::new(0))?;
    nw.connect_both(mac_1, Port::new(1), mac_b, Port::new(0))?;
    nw.connect_both(mac_1, Port::new(2), mac_r, Port::new(0))?;
    nw.connect_both(mac_r, Port::new(1), mac_s, Port::new(1))?;
    nw.connect_both(mac_3, Port::new(0), mac_c, Port::new(0))?;
    nw.connect_both(mac_3, Port::new(1), mac_d, Port::new(0))?;
    nw.connect_both(mac_3, Port::new(2), mac_s, Port::new(0))?;

    nw.run(550).unwrap();
    let d = nw.get_device(mac_a)?;
    let d = d.as_any().downcast_ref::<IpHost>().unwrap();
    let rlogs = d.get_rlog();
    assert_eq!(1, rlogs.len());
    let plog: &IP = match rlogs[0].p {
        NetworkProtocol::IP(ref p) => p,
        _ => panic!("")
    };
    assert_eq!(plog.dst, addr_1a);
    assert_eq!(plog.src, addr_3d);
    Ok(())
}

pub fn run_unreachable() -> Res<()> {
    println!("netwl sample. unreachable");
    crate::output::set_level(crate::output::Level::Frame);
    let subnet_mask = SubnetMask::new(24);

    let addr_1a = IpAddr::new(0x0a01_0001);
    let addr_1b = IpAddr::new(0x0a01_0002);
    let addr_1r = IpAddr::new(0x0a01_0003);
    let addr_2r = IpAddr::new(0x0a02_0001);
    let addr_2s = IpAddr::new(0x0a02_0002);
    let addr_3c = IpAddr::new(0x0a03_0001);
    let addr_3d = IpAddr::new(0x0a03_0002);
    let addr_3s = IpAddr::new(0x0a03_0003);

    let mac_a = Mac::new(761);
    let mac_b = Mac::new(762);
    let mac_c = Mac::new(763);
    let mac_d = Mac::new(764);
    let mac_r = Mac::new(765);
    let mac_s = Mac::new(766);
    let mac_1 = Mac::new(767);
    let mac_3 = Mac::new(768);

    let mut host_a = IpHost::build_echo(mac_a, "host1a", addr_1a, subnet_mask);
    let host_b = IpHost::build_echo(mac_b, "host1b", addr_1b, subnet_mask);
    let host_c = IpHost::build_echo(mac_c, "host3c", addr_3c, subnet_mask);
    let mut host_d = IpHost::build_echo(mac_d, "host3d", addr_3d, subnet_mask);
    let mut router_r = BaseIpDevice::new_router(mac_r, "routeR", vec![addr_1r, addr_2r], subnet_mask);
    let mut router_s = BaseIpDevice::new_router(mac_s, "routeS", vec![addr_3s, addr_2s], subnet_mask);
    let switch_1 = EthernetSwitch::build_switch(mac_1, "switch1", 3);
    let switch_3 = EthernetSwitch::build_switch(mac_3, "switch3", 3);

    let ip0 = IP::new_byte(addr_1a, addr_3d, vec![0x01, 0x02]);
    host_a.add_schedule(0, NetworkProtocol::IP(ip0));
    host_a.add_arp_entry(addr_1r, mac_r)?;
    host_d.add_arp_entry(addr_3s, mac_s)?;
    router_r.add_arp_entry(addr_1a, mac_a)?;
    router_r.add_arp_entry(addr_2s, mac_s)?;
    router_s.add_arp_entry(addr_3d, mac_d)?;
    router_s.add_arp_entry(addr_2r, mac_r)?;

    let nw_part = addr_3s.nw(subnet_mask);
    host_a.add_route_entry(nw_part, addr_1r)?;
    // router_r.add_route_entry(nw_part, addr_2s)?;
    router_s.add_route_entry(nw_part, addr_3d)?;

    let nw1_part = addr_1r.nw(subnet_mask);
    host_d.add_route_entry(nw1_part, addr_3s)?;
    router_s.add_route_entry(nw1_part, addr_2r)?;

    let mut nw = Network::new(
        vec![host_a, host_b, host_c, host_d, router_r, router_s, switch_1, switch_3],
        vec![]
    );
    nw.connect_both(mac_1, Port::new(0), mac_a, Port::new(0))?;
    nw.connect_both(mac_1, Port::new(1), mac_b, Port::new(0))?;
    nw.connect_both(mac_1, Port::new(2), mac_r, Port::new(0))?;
    nw.connect_both(mac_r, Port::new(1), mac_s, Port::new(1))?;
    nw.connect_both(mac_3, Port::new(0), mac_c, Port::new(0))?;
    nw.connect_both(mac_3, Port::new(1), mac_d, Port::new(0))?;
    nw.connect_both(mac_3, Port::new(2), mac_s, Port::new(0))?;

    let res = nw.run(400);
    match &res {
        Err(e) => println!("{}", e),
        _ => panic!("expect error"),
    }

    res
}

pub fn run_test_router_arp() -> Res<()> {
    crate::output::set_level(crate::output::Level::Frame);
    let subnet_mask = SubnetMask::new(24);
    let addr_a = IpAddr::new(0x0a00_0001);
    let addr_b = IpAddr::new(0x0a00_0002);
    let addr_r = IpAddr::new(0x0a00_0003);

    let mac_a = Mac::new(761);
    let mac_b = Mac::new(762);
    let mac_s = Mac::new(763);
    let mac_r = Mac::new(764);

    let mut host_a = IpHost::build_echo(mac_a, "hostA", addr_a, subnet_mask);
    let arp0 = ARP::new_request(mac_a, addr_a, addr_r);
        // host_a.get_mac(), host_a.get_ip_addr(Port::new(0)).unwrap(), addr_r);
    host_a.add_schedule(0, NetworkProtocol::ARP(arp0));
    // host_a.add_arp_entry(addr_b, mac_b)?;

    let host_b = IpHost::build_echo(mac_b, "hostB", addr_b, subnet_mask);
    let switch = EthernetSwitch::build_switch(mac_s, "switch", 3);
    let router = BaseIpDevice::new_router(mac_r, "router", vec![addr_r], subnet_mask);

    let mut nw = Network::new(
        vec![host_a, host_b, switch, router],
        vec![]
    );
    nw.connect_both(mac_s, Port::new(0), mac_a, Port::new(0))?;
    nw.connect_both(mac_s, Port::new(1), mac_b, Port::new(0))?;
    nw.connect_both(mac_s, Port::new(2), mac_r, Port::new(0))?;

    nw.run(250).unwrap();

    let d = nw.get_device(mac_a).unwrap();
    let d = d.as_any().downcast_ref::<IpHost>().unwrap();
    let arp_table = d.get_arp_table();
    assert_eq!(1, arp_table.len());
    let (ipaddr, mac ) = arp_table.iter().next().unwrap();
    // for (ipaddr, mac) in arp_table.iter() {
    //     println!("arp table: {:} : {:}", ipaddr, mac.value);
    // }
    // println!("arp table: {:?}", d.base.arp_table);
    println!("arp: {} : {}", ipaddr, mac.value);
    assert_eq!(addr_r, *ipaddr);
    assert_eq!(mac_r, *mac);
    Ok(())        
}

#[cfg(test)]
mod tests {
    use super::super::types::Error;

    use super::*;

    #[test]
    fn test_host_host() {
        run_host_host().unwrap();
    }

    #[test]
    fn test_2host_1router() {
        run_2host_1router().unwrap();
    }

    #[test]
    fn test_2router() {
        run_2router().unwrap();
    }

    #[test]
    fn test_unreachable() {
        let nw = run_unreachable();
        match nw {
            Err(Error::IpUnreashcable { code, msg: _msg }) => assert_eq!(1, code),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_router_arp() {
        run_test_router_arp().unwrap();
    }
}