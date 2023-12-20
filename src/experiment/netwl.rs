pub mod ip_addr;
pub mod ip;
pub mod arp;
pub mod network_protocol;
pub mod ip_device;

use crate::experiment::linkl::EthernetDevice;

use super::types::{Port, Mac, Res, Error};
use super::physl::{UpdateContext, Device, Network};
use super::linkl::EthernetFrame;

use ip_addr::*;
use ip::*;
use arp::*;
use network_protocol::*;
use ip_device::*;

// impl NetworkProtocol {
//     pub fn encode(&self) -> Vec<u8> {
//         match self {
//             NetworkProtocol::IP(ip) => ip.encode(),
//             NetworkProtocol::ARP(arp) => arp.encode(),
//         }
//     }
// }
type PayloadFn = dyn FnMut(&Vec<u8>) -> Option<Vec<u8>>;

pub struct IpHost {
    base: BaseIpDevice,
    schedules: Vec<(usize, NetworkProtocol)>,
    payload_fn: Box<PayloadFn>,
    slog: Vec<NetworkLog>,
    rlog: Vec<NetworkLog>,
    arp_rep_waitings: Vec<IP>,
}

impl IpHost {
    pub fn new_echo(mac: Mac, name: &str, ip_addr: IpAddr, subnet_mask: SubnetMask) -> Box<IpHost> {
        let f = |x: &Vec<u8>| {
            Some(x.clone())
        };
        let host = IpHost {
            base: BaseIpDevice::new(mac, name, vec![ip_addr.clone()], subnet_mask),
            schedules: vec![],
            payload_fn: Box::new(f),
            slog: Vec::new(),
            rlog: Vec::new(),
            arp_rep_waitings: Vec::new(),
        };
        Box::new(host)
    }

    pub fn add_schedule(&mut self, t: usize, p: NetworkProtocol) {
        self.schedules.push((t, p));
    }

    pub fn add_arp_entry(&mut self, ip_addr: IpAddr, mac: Mac) -> Res<()> {
        self.base.add_arp_entry(ip_addr, mac)
    }

    pub fn get_ip_addr(&self) -> IpAddr {
        self.base.ip_addr_ports[0].0
    }

    pub fn add_route_entry(&mut self, nw_part: NetworkPart, ip_addr: IpAddr) -> Res<()> {
        self.base.add_route_entry(nw_part, ip_addr)
    }

    fn get_rlog(&self) -> &Vec<NetworkLog> {
        &self.rlog
    }

    fn push_sbuf(&mut self, p: NetworkProtocol, ctx: &UpdateContext) -> Res<()> {
        self.slog.push(NetworkLog { t: ctx.t, p: p.clone() });
        self.base.push_sbuf(p, ctx)
    }

    fn update_ip(&mut self, ip: &IP, ctx: &UpdateContext) -> Res<()> {
        if ip.dst == self.get_ip_addr() {
            // FIXME
            // prepare respnse and send to src
            match &ip.payload {
                IpPayload::Bytes(xs) => {
                    let payload = (self.payload_fn)(xs);
                    match payload {
                        None => {}
                        Some(payload) => {
                            let ip = IP::new_byte(ip.dst, ip.src, payload);
                            let ip = NetworkProtocol::IP(ip);
                            self.push_sbuf(ip, ctx)?;
                        }
                    }
                },
                IpPayload::ICMP { ty, code } if (*ty == 3) => {
                    return Err(Error::IpUnreashcable { code: *code, msg: "".to_string() });
    
                    // FIXME: sending ARP is not correct
                    // let p = self.slog.iter()
                    // .filter_map( |x| {
                    //     match x.p {
                    //         NetworkProtocol::IP(ref p) => Some(p),
                    //         _ => None,
                    //     }
                    // })
                    // .find(|_p| {
                    //     // FIXME: filter by ID
                    //     // p.dst == ip.src && p.src == ip.dst                        
                    //     true
                    // });
                    // if let Some(p) = p {
                    //     let arp = ARP::new_request(self.get_mac(), self.get_ip_addr(), p.dst);
                    //     let arp = NetworkProtocol::ARP(arp);
                    //     self.push_sbuf(arp, ctx)?;
                    // } else {
                    //     // println!("slog={:?}", self.slog);
                    //     panic!("ICMP(unreachable) received but original IP does not found");
                    // }
                },
                _ => {
                    panic!("not implemented ip payload. payload={:?}", ip.payload);
                }
            }
        }
    Ok(())

    // let f = &mut |p: NetworkProtocol| {
    //     match p {
    //         NetworkProtocol::IP(ip) => {
    //             let ip = IP::new(ip.dst, ip.src);
    //             let ip = NetworkProtocol::IP(ip);
    //             vec![ip]
    //         },
    //         NetworkProtocol::ARP(_arp) => {
    //             vec![]
    //         },
    //     }
    // };
    // self.base.update(ctx, f)
}

    fn update_arp(&mut self, arp: &ARP, ctx: &UpdateContext) -> Res<()> {
        if arp.target_ipaddr == self.get_ip_addr() {
            match arp.opcode {
                1  => {
                    let arp = arp.reply(self.get_mac());
                    let arp = NetworkProtocol::ARP(arp);
                    self.base.push_sbuf(arp, ctx)?;
                },
                2 => {
                    self.base.add_arp_entry(arp.sender_ipaddr, arp.sender_mac)?;
                    if let Some((i, ip))= self.arp_rep_waitings.iter().enumerate().find(|(_t, x)| x.dst == arp.sender_ipaddr) {
                        let ip = NetworkProtocol::IP(ip.clone());
                        self.base.push_sbuf(ip, ctx)?;
                        self.arp_rep_waitings.remove(i);
                    }
                },
                _ => panic!("invalid arp opcode"),
            }
        } else {
            // do nothing
        }
        Ok(())
    }
}

impl Device for IpHost {
    fn base(&self) -> &super::physl::BaseDevice {
        &self.base.base.base
    }

    fn base_mut(&mut self) -> &mut super::physl::BaseDevice {
        &mut self.base.base.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) -> Res<()> {
        for idx in 0..self.schedules.len() {
            let (t, p) = &self.schedules[idx];
            if *t == ctx.t {
                let p = p.clone();
                self.push_sbuf(p, ctx)?;
            }
        }
        while let Some(p) = self.base.pop_rbuf(ctx)? {
            self.rlog.push(NetworkLog { t: ctx.t, p: p.clone() });

            match p {
                NetworkProtocol::IP(ip) => self.update_ip(&ip, ctx)?,
                NetworkProtocol::ARP(arp) => self.update_arp(&arp, ctx)?,
            };
        }
        Ok(())
    }
}


#[derive(Debug)]
struct NetworkLog {
    t: usize,
    p: NetworkProtocol,
}

pub struct Router {
    base: BaseIpDevice
}

impl Router {
    pub fn box_new(mac: Mac, name: &str, ip_addr_list: Vec<IpAddr>, subnet_mask: SubnetMask) -> Box<Router> {
        let router = Router {
            base: BaseIpDevice::new(mac, name, ip_addr_list, subnet_mask),
        };
        Box::new(router)
    }

    fn push_sbuf(&mut self, p: NetworkProtocol, ctx: &UpdateContext) -> Res<()> {
        match p {
            NetworkProtocol::IP(ip) => self.push_sbuf_ip(ip, ctx),
            NetworkProtocol::ARP(arp) => self.push_sbuf_arp(arp, ctx),
        }
    }

    fn push_sbuf_ip(&mut self, ip: IP, ctx: &UpdateContext) -> Res<()> {
        // FIXME: how to determine src_ip here ?
        // FIXME: port
        // let port_receive = Port::new(0);
        let p = ip;
        let dst_nw_part = NetworkPart::new(p.dst, self.base.subnet_mask);
        let dst_mac = if let Some(port) = self.base.find_port(&dst_nw_part) {
            // dst is in same network
            // if port.value == port_receive.value {
            //     // drop frame
            //     return Ok(())
            // } else {
                if let Some(dst_mac) = self.base.arp_table.get(&p.dst) {
                    self.base.base.add_forwarding_table(*dst_mac, port);
                    dst_mac
                } else {
                    panic!("failed to find in arp table. ip.dst={}", p.dst);
                // }
            }

        } else if let Some(dst_ip) = self.base.routing_table.get(&dst_nw_part) {
            // dst is other network

            if let Some(dst_mac) = self.base.arp_table.get(dst_ip) {
                // update forwarding table
                let nw_part = dst_ip.nw(self.base.subnet_mask);
                if let Some(port) = self.base.find_port(&nw_part) {
                    self.base.base.add_forwarding_table(*dst_mac, port)
                } else {
                    panic!("failed to find network part. nw={}", nw_part);
                }

                dst_mac
            } else {
                // FIXME: send ICMP (unreachable)
                // self.push_sbuf(ICMP(src: self.src, dst: ip.src, ICMPType::Unreachable), ctx)?;
                panic!("failed to find in arp table");
            }
        } else {
            // type=3 => destination unreachable
            // code=0 => network unreachable
            // FIXME: choses src collect
            let icmp = IP::new_icmp( self.base.ip_addr_ports[0].0, p.src, 3, 0 );
            let icmp = NetworkProtocol::IP(icmp);
            self.push_sbuf(icmp, ctx)?;
            return Ok(())
        };

        let disp = crate::output::is_frame_level();
        if disp {
            print!("{:>3}: ", ctx.t);
            // FIXME: print ip address correct
            println!("{}({}): send   : {:}", 
                     self.get_name(), 
                     self.base.ip_addr_ports[0].0, p);
        }

        let src_mac = self.get_mac();
        let payload = p.encode();
        let ethertype = 0x0800;
        let frame = EthernetFrame::new(*dst_mac, src_mac, ethertype, payload);
        self.base.base.push_sbuf(frame, ctx);

        Ok(())
    }

    fn push_sbuf_arp(&mut self, arp: ARP, ctx: &UpdateContext) -> Res<()> {
        // FIXME: this code should be in Update
        // FIXME: duplicated code
        if arp.target_ipaddr == self.base.ip_addr_ports[0].0 {
            match arp.opcode {
                1  => { // FIXME: request
                    let arp = arp.reply(self.get_mac());
                    let arp = NetworkProtocol::ARP(arp);
                    self.base.push_sbuf(arp, ctx)?;
                },
                2 => { // FIXME: reply
                    self.base.add_arp_entry(arp.sender_ipaddr, arp.sender_mac)?;
                },
                _ => panic!("invalid arp opcode"),
            }
        } else {
            // do nothing
        }

        Ok(())
    }

    pub fn add_arp_entry(&mut self, ip_addr: IpAddr, mac: Mac) -> Res<()> {
        self.base.add_arp_entry(ip_addr, mac)
    }

    pub fn add_route_entry(&mut self, nw_part: NetworkPart, ip_addr: IpAddr) -> Res<()> {
        self.base.add_route_entry(nw_part, ip_addr)
    }
}

impl Device for Router {
    fn base(&self) -> &super::physl::BaseDevice {
        &self.base.base.base
    }

    fn base_mut(&mut self) -> &mut super::physl::BaseDevice {
        &mut self.base.base.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) -> Res<()> {
        // FIXME:
        // 1. nw_part(dst) == self.nw_part => drop
        // 2. nw_part(dst) == other.nw_part (in RoutingTable) => send to other
        // 3. otherwise => panic

        while let Some(p) = self.base.pop_rbuf(ctx)? {
            self.push_sbuf(p, ctx)?;
        }
        Ok(())
    }
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
    // let mut host0 = IpHost::new_echo(mac0, "host1", addr0, subnet_mask);
    let mut host0 = IpDevice::new_host_echo(mac0, "host1", addr0, subnet_mask);
    host0.add_schedule(0, NetworkProtocol::IP(ip0));
    host0.add_arp_entry(addr1, mac1)?;
    let host1 = IpHost::new_echo(mac1, "host2", addr1, subnet_mask);

    let mut nw = Network::new(
        vec![host0, host1],
        vec![]
    );
    nw.connect_both(mac0, port0, mac1, port1).unwrap();
    nw.run(100).unwrap();
    let d = nw.get_device(mac0).unwrap();
    let d = d.as_any().downcast_ref::<IpDevice>().unwrap();
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

    let mut host_a = IpHost::new_echo(mac_a, "hostA", addr_a, subnet_mask);
    let ip0 = IP::new_byte(addr_a, addr_b, vec![0x01, 0x02]);
    host_a.add_schedule(0, NetworkProtocol::IP(ip0));
    host_a.add_arp_entry(addr_b, mac_b)?;

    let host_b = IpHost::new_echo(mac_b, "hostB", addr_b, subnet_mask);
    let switch = EthernetDevice::build_switch(mac_s, "switch", 3);
    let router = Router::box_new(mac_r, "router", vec![addr_r], subnet_mask);

    let mut nw = Network::new(
        vec![host_a, host_b, switch, router],
        vec![]
    );
    nw.connect_both(mac_s, Port::new(0), mac_a, Port::new(0))?;
    nw.connect_both(mac_s, Port::new(1), mac_b, Port::new(0))?;
    nw.connect_both(mac_s, Port::new(2), mac_r, Port::new(0))?;

    nw.run(100).unwrap();
    // let d = nw.get_device(mac0).unwrap();
    // let d = d.as_any().downcast_ref::<IpHost>().unwrap();
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

    let mut host_a = IpHost::new_echo(mac_a, "host1a", addr_1a, subnet_mask);
    let host_b = IpHost::new_echo(mac_b, "host1b", addr_1b, subnet_mask);
    let host_c = IpHost::new_echo(mac_c, "host3c", addr_3c, subnet_mask);
    let mut host_d = IpHost::new_echo(mac_d, "host3d", addr_3d, subnet_mask);
    let mut router_r = Router::box_new(mac_r, "routeR", vec![addr_1r, addr_2r], subnet_mask);
    let mut router_s = Router::box_new(mac_s, "routeS", vec![addr_3s, addr_2s], subnet_mask);
    let switch_1 = EthernetDevice::build_switch(mac_1, "switch1", 3);
    let switch_3 = EthernetDevice::build_switch(mac_3, "switch3", 3);

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

    let mut host_a = IpHost::new_echo(mac_a, "host1a", addr_1a, subnet_mask);
    let host_b = IpHost::new_echo(mac_b, "host1b", addr_1b, subnet_mask);
    let host_c = IpHost::new_echo(mac_c, "host3c", addr_3c, subnet_mask);
    let mut host_d = IpHost::new_echo(mac_d, "host3d", addr_3d, subnet_mask);
    let mut router_r = Router::box_new(mac_r, "routeR", vec![addr_1r, addr_2r], subnet_mask);
    let mut router_s = Router::box_new(mac_s, "routeS", vec![addr_3s, addr_2s], subnet_mask);
    let switch_1 = EthernetDevice::build_switch(mac_1, "switch1", 3);
    let switch_3 = EthernetDevice::build_switch(mac_3, "switch3", 3);

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

pub fn run_test_arp() -> Res<()> {
    crate::output::set_level(crate::output::Level::Frame);
    let subnet_mask = SubnetMask::new(24);
    let addr_a = IpAddr::new(0x0a00_0001);
    let addr_b = IpAddr::new(0x0a00_0002);
    let addr_r = IpAddr::new(0x0a00_0003);

    let mac_a = Mac::new(761);
    let mac_b = Mac::new(762);
    let mac_s = Mac::new(763);
    let mac_r = Mac::new(764);

    let mut host_a = IpHost::new_echo(mac_a, "hostA", addr_a, subnet_mask);
    let arp0 = ARP::new_request(host_a.get_mac(), host_a.get_ip_addr(), addr_r);
    host_a.add_schedule(0, NetworkProtocol::ARP(arp0));
    // host_a.add_arp_entry(addr_b, mac_b)?;

    let host_b = IpHost::new_echo(mac_b, "hostB", addr_b, subnet_mask);
    let switch = EthernetDevice::build_switch(mac_s, "switch", 3);
    let router = Router::box_new(mac_r, "router", vec![addr_r], subnet_mask);

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
    assert_eq!(1, d.base.arp_table.len());
    let (ipaddr, mac ) = d.base.arp_table.iter().next().unwrap();
    for (ipaddr, mac) in d.base.arp_table.iter() {
        println!("arp table: {:} : {:}", ipaddr, mac.value);
    }
    // println!("arp table: {:?}", d.base.arp_table);
    assert_eq!(addr_r, *ipaddr);
    assert_eq!(mac_r, *mac);
    Ok(())        
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_host() {
        run_host_host().unwrap();
    }

    #[test]
    fn test_2router() {
        run_2router().unwrap();
    }

    #[test]
    fn test_unreachable() {
        let nw = run_unreachable();
        match nw {
            Err(Error::IpUnreashcable { code, msg: _msg }) => assert_eq!(0, code),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_arp() {
        run_test_arp().unwrap();
    }
}