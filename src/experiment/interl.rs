pub mod ip_addr;
pub mod ip;
pub mod arp;

use std::collections::{VecDeque, HashMap};

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
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseIpDevice {
        BaseIpDevice {
            rbuf: VecDeque::new(),
            // sbuf: VecDeque::new(),
            base: BaseEthernetDevice::new(mac, name, num_ports),
            subnet_mask: SubnetMask::new(24),
            ip_addr_ports: vec![],
            routing_table: HashMap::new(),
            arp_table: HashMap::new(),
        }
    }

    pub fn pop_rbuf(&mut self) -> Res<Option<NetworkProtocol>> {
        // FIXME: add output
        // FIXME: update route table
        while let Some(frame) = self.base.rbuf.pop_front() {
            if let Ok(ip) = IP::decode(&frame.payload) {
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

        // FIXME: how to determine src_ip here ?

        let dst_nw_part = NetworkPart::build(p.dst, self.subnet_mask);
        let dst_mac = if let Some(port) = self.find_port(&dst_nw_part) {
            // dst is in same network
            if let Some(dst_mac) = self.arp_table.get(&p.dst) {
                self.base.add_forwarding_table(*dst_mac, port);
                dst_mac
            } else {
                panic!("failed to find in arp table");
            }
        } else if let Some(dst_ip) = self.routing_table.get(&dst_nw_part) {
            if let Some(dst_mac) = self.arp_table.get(dst_ip) {
                dst_mac
            } else {
                panic!("failed to find in arp table");
            }
        } else {
            panic!("failed to find dst_nw_part. dst_nw_part={:X}", dst_nw_part.get_value());
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
            NetworkPart::build(*ip_addr, self.subnet_mask) == *nw_part
        })
        .map(|(_, port)| *port)
    }

    pub fn update(&mut self, ctx: &UpdateContext, f: &mut Handler) -> Res<()> {
        while let Some(p) = self.pop_rbuf()? {
            let response_list = f(p);
            for p in response_list {
                self.push_sbuf(p, ctx)?;
            }
        }
        Ok(())
    }
}

type Handler = dyn FnMut(NetworkProtocol) -> Vec<NetworkProtocol>;

pub struct IpHost {
    base: BaseIpDevice,
    schedules: Vec<(usize, NetworkProtocol)>,
    // receives: Vec<(usize, NetworkProtocol)>,
}

impl IpHost {
    pub fn build(mac: Mac, name: &str, num_ports: usize) -> Box<IpHost> {
        let host = IpHost {
            base: BaseIpDevice::new(mac, name, num_ports),
            schedules: vec![],
            // receives: vec![],
        };
        Box::new(host)
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
        let f = &mut |p: NetworkProtocol| {
            // self.receives.push((ctx.t, p.clone()));
            vec![p]
        };
        self.base.update(ctx, f)
    }
}

pub fn run_sample() {
    let mac0 = Mac::new(0);
    let mac1 = Mac::new(1);
    let port0 = Port::new(0);
    let port1 = Port::new(0);
    let host0 = IpHost::build(mac0, "host1", 1);
    let host1 = IpHost::build(mac1, "host2", 1);

    let mut nw = Network::new(
        vec![host0, host1],
        vec![]
    );
    nw.connect_both(mac0, port0, mac1, port1).unwrap();
    nw.run(10).unwrap();
    let d = nw.get_device(mac0).unwrap();
    let d = d.as_any().downcast_ref::<IpHost>().unwrap();
    println!("{}", d.get_name());    

}

