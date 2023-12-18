pub mod ip_addr;
pub mod ip;
pub mod arp;

use std::collections::{VecDeque, HashMap};

use crate::experiment::linkl::EthernetDevice;

use super::types::{Port, Mac, Res, Error};
use super::physl::{UpdateContext, Device, Network};
use super::linkl::{BaseEthernetDevice, EthernetFrame};

use ip_addr::*;
use ip::*;
use arp::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkProtocol {
    IP(IP),
    ARP(ARP),
}

// impl NetworkProtocol {
//     pub fn encode(&self) -> Vec<u8> {
//         match self {
//             NetworkProtocol::IP(ip) => ip.encode(),
//             NetworkProtocol::ARP(arp) => arp.encode(),
//         }
//     }
// }

pub struct BaseIpDevice {
    rbuf: VecDeque<NetworkProtocol>,
    // sbuf: VecDeque<Protocol>,
    pub base: BaseEthernetDevice,

    subnet_mask: SubnetMask,
    ip_addr_ports: Vec<(IpAddr, Port)>,
    routing_table: HashMap<NetworkPart, IpAddr>,
    arp_table: HashMap<IpAddr, Mac>,
}

impl BaseIpDevice {
    pub fn new(mac: Mac, name: &str, ip_addr_list: Vec<IpAddr>) -> BaseIpDevice {
        let ip_addr_ports: Vec<(IpAddr, Port)> = ip_addr_list
            .into_iter()
            .enumerate()
            .map(|(i, ip_addr)| {
                (ip_addr, Port::new(i as u32))
            })
            .collect();
        // let num_ports = ip_addr_ports.len();
        BaseIpDevice {
            rbuf: VecDeque::new(),
            // sbuf: VecDeque::new(),
            // FIXME: support hub
            base: BaseEthernetDevice::new(mac, name, ip_addr_ports.len()),
            subnet_mask: SubnetMask::new(24),
            ip_addr_ports: ip_addr_ports,
            routing_table: HashMap::new(),
            arp_table: HashMap::new(),
        }
    }

    pub fn pop_rbuf(&mut self, ctx: &UpdateContext) -> Res<Option<NetworkProtocol>> {
        let disp = crate::output::is_frame_level();

        // FIXME: update route table
        while let Some(frame) = self.base.pop_rbuf(ctx) {

            if let Ok(ip) = IP::decode(&frame.payload) {
                if disp {
                    print!("{:>3}: ", ctx.t);
                    // FIXME: print ip address correct
                    println!("{}({}): receive: {:}", 
                             self.base.base.get_name(), 
                             self.ip_addr_ports[0].0, ip);
                }
                self.add_arp_entry(ip.src, frame.src)?;

                self.rbuf.push_back(NetworkProtocol::IP(ip));
            } else if let Ok(arp) = ARP::decode(&frame.payload) {
                self.rbuf.push_back(NetworkProtocol::ARP(arp));
            } else {
                return Err(Error::InvalidBytes { msg: "IP".to_string() })
            }
        }
        Ok(self.rbuf.pop_front())
    }

    pub fn push_sbuf(&mut self, p: NetworkProtocol, ctx: &UpdateContext) -> Res<()> {
        let p = match p {
            NetworkProtocol::IP(ip) => ip,
            NetworkProtocol::ARP(_) => panic!("ARP is not supported yet"),
        };

        let disp = crate::output::is_frame_level();
        if disp {
            print!("{:>3}: ", ctx.t);
            // FIXME: print ip address correct
            println!("{}({}): send   : {:}", 
                     self.base.base.get_name(), 
                     self.ip_addr_ports[0].0, p);
        }

        // FIXME: how to determine src_ip here ?

        let dst_nw_part = NetworkPart::new(p.dst, self.subnet_mask);
        let dst_mac = if let Some(port) = self.find_port(&dst_nw_part) {
            // dst is in same network
            // FIXME: check frame come from the same network
            if let Some(dst_mac) = self.arp_table.get(&p.dst) {
                self.base.add_forwarding_table(*dst_mac, port);
                dst_mac
            } else {
                panic!("failed to find in arp table. ip.dst={}", p.dst);
            }
        } else if let Some(dst_ip) = self.routing_table.get(&dst_nw_part) {
            if let Some(dst_mac) = self.arp_table.get(dst_ip) {
                dst_mac
            } else {
                panic!("failed to find in arp table");
            }
        } else {
            panic!("failed to find dst_nw_part. dst_nw_part={:}", dst_nw_part);
        };

        let src_mac = self.base.base.get_mac();
        let payload = p.encode();
        let ethertype = payload.len() as u16;
        let frame = EthernetFrame::new(*dst_mac, src_mac, ethertype, payload);
        self.base.push_sbuf(frame, ctx);

        Ok(())
    }

    fn find_port(&self, nw_part: &NetworkPart) -> Option<Port> {
        self.ip_addr_ports
        .iter()
        .find(|(ip_addr, _)| {
            NetworkPart::new(*ip_addr, self.subnet_mask) == *nw_part
        })
        .map(|(_, port)| *port)
    }

    pub fn add_arp_entry(&mut self, ip_addr: IpAddr, mac: Mac) -> Res<()> {
        self.arp_table.insert(ip_addr, mac);
        Ok(())
    }

    // pub fn update(&mut self, ctx: &UpdateContext, f: &mut Handler) -> Res<()> {
    //     while let Some(p) = self.pop_rbuf(ctx)? {
    //         let response_list = f(p);
    //         for p in response_list {
    //             self.push_sbuf(p, ctx)?;
    //         }
    //     }
    //     Ok(())
    // }
}

type PayloadFn = dyn FnMut(&Vec<u8>) -> Option<Vec<u8>>;

pub struct IpHost {
    base: BaseIpDevice,
    schedules: Vec<(usize, NetworkProtocol)>,
    payload_fn: Box<PayloadFn>,
    // receives: Vec<(usize, NetworkProtocol)>,
}

impl IpHost {
    pub fn new_echo(mac: Mac, name: &str, ip_addr: IpAddr) -> Box<IpHost> {
        let f = |x: &Vec<u8>| {
            Some(x.clone())
        };
        let host = IpHost {
            base: BaseIpDevice::new(mac, name, vec![ip_addr.clone()]),
            schedules: vec![],
            payload_fn: Box::new(f),
            // receives: vec![],
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
        for (t, p) in self.schedules.iter() {
            if *t == ctx.t {
                self.base.push_sbuf(p.clone(), ctx)?;
            }
        }
        while let Some(p) = self.base.pop_rbuf(ctx)? {
            let ip = match p {
                NetworkProtocol::IP(ip) => ip,
                _ => panic!("not supported yet"),
            };

            if ip.dst == self.get_ip_addr() {
                // FIXME
                // prepare respnse and send to src
                let payload_received = vec![0x00];
                let payload = (self.payload_fn)(&payload_received);
                match payload {
                    None => {}
                    Some(_payload) => {
                        // FIXME: push payload
                        let ip = IP::new(ip.dst, ip.src);
                        let ip = NetworkProtocol::IP(ip);
                        self.base.push_sbuf(ip, ctx)?;
                    }
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
}

pub struct Router {
    base: BaseIpDevice
}

impl Router {
    pub fn box_new(mac: Mac, name: &str, ip_addr_list: Vec<IpAddr>) -> Box<Router> {
        let router = Router {
            base: BaseIpDevice::new(mac, name, ip_addr_list),
        };
        Box::new(router)
    }

    fn push_sbuf(&mut self, p: NetworkProtocol, ctx: &UpdateContext) -> Res<()> {
        let p = match p {
            NetworkProtocol::IP(ip) => ip,
            NetworkProtocol::ARP(_) => panic!("ARP is not supported yet"),
        };

        // FIXME: how to determine src_ip here ?
        // FIXME: port
        let port_receive = Port::new(0);
        let dst_nw_part = NetworkPart::new(p.dst, self.base.subnet_mask);
        let dst_mac = if let Some(port) = self.base.find_port(&dst_nw_part) {
            // dst is in same network
            if port.value == port_receive.value {
                // drop frame
                return Ok(())
            } else {
                if let Some(dst_mac) = self.base.arp_table.get(&p.dst) {
                    self.base.base.add_forwarding_table(*dst_mac, port);
                    dst_mac
                } else {
                    panic!("failed to find in arp table. ip.dst={}", p.dst);
                }
            }

        } else if let Some(dst_ip) = self.base.routing_table.get(&dst_nw_part) {
            // dst is other network
            if let Some(dst_mac) = self.base.arp_table.get(dst_ip) {
                dst_mac
            } else {
                panic!("failed to find in arp table");
            }
        } else {
            panic!("failed to find dst_nw_part. dst_nw_part={:}", dst_nw_part);
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
        let ethertype = payload.len() as u16;
        let frame = EthernetFrame::new(*dst_mac, src_mac, ethertype, payload);
        self.base.base.push_sbuf(frame, ctx);

        Ok(())
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
    let addr0 = IpAddr::new(0x0a00_0001);
    let addr1 = IpAddr::new(0x0a00_0002);
    let ip0 = IP::new(addr0, addr1);
    let mac0 = Mac::new(761);
    let mac1 = Mac::new(762);
    let port0 = Port::new(0);
    let port1 = Port::new(0);
    let mut host0 = IpHost::new_echo(mac0, "host1", addr0);
    host0.add_schedule(0, NetworkProtocol::IP(ip0));
    host0.add_arp_entry(addr1, mac1)?;
    let host1 = IpHost::new_echo(mac1, "host2", addr1);

    let mut nw = Network::new(
        vec![host0, host1],
        vec![]
    );
    nw.connect_both(mac0, port0, mac1, port1).unwrap();
    nw.run(100).unwrap();
    let d = nw.get_device(mac0).unwrap();
    let d = d.as_any().downcast_ref::<IpHost>().unwrap();
    println!("{}", d.get_name());    

    Ok(())
}

pub fn run_2host_1router() -> Res<()> {
    crate::output::set_level(crate::output::Level::Frame);
    let addr_a = IpAddr::new(0x0a00_0001);
    let addr_b = IpAddr::new(0x0a00_0002);
    let addr_r = IpAddr::new(0x0a00_0003);

    let mac_a = Mac::new(761);
    let mac_b = Mac::new(762);
    let mac_s = Mac::new(763);
    let mac_r = Mac::new(764);

    let mut host_a = IpHost::new_echo(mac_a, "hostA", addr_a);
    let ip0 = IP::new(addr_a, addr_b);
    host_a.add_schedule(0, NetworkProtocol::IP(ip0));
    host_a.add_arp_entry(addr_b, mac_b)?;

    let host_b = IpHost::new_echo(mac_b, "hostB", addr_b);
    let switch = EthernetDevice::build_switch(mac_s, "switch", 3);
    let router = Router::box_new(mac_r, "router", vec![addr_r]);

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
