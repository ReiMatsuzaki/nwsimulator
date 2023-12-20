use std::collections::{VecDeque, HashMap};
use super::super::types::*;
use super::super::physl::UpdateContext;
use super::super::linkl::{BaseEthernetDevice, EthernetFrame};
use super::network_protocol::*;
use super::ip::*;
use super::arp::*;
use super::ip_addr::*;

pub struct BaseIpDevice {
    rbuf: VecDeque<NetworkProtocol>,
    // sbuf: VecDeque<Protocol>,
    pub base: BaseEthernetDevice,

    pub subnet_mask: SubnetMask,
    pub ip_addr_ports: Vec<(IpAddr, Port)>,
    pub routing_table: HashMap<NetworkPart, IpAddr>,
    pub arp_table: HashMap<IpAddr, Mac>,
}

impl BaseIpDevice {
    pub fn new(mac: Mac, name: &str, ip_addr_list: Vec<IpAddr>, subnet_mask: SubnetMask) -> BaseIpDevice {
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
            subnet_mask,
            ip_addr_ports: ip_addr_ports,
            routing_table: HashMap::new(),
            arp_table: HashMap::new(),
        }
    }

    pub fn pop_rbuf(&mut self, ctx: &UpdateContext) -> Res<Option<NetworkProtocol>> {
        let disp = crate::output::is_frame_level();

        while let Some(frame) = self.base.pop_rbuf(ctx) {

            let p = match frame.ethertype {
                0x0800 => { 
                    // IPv4
                    let ip = IP::decode(&frame.payload)?;
                    // FIXME: update here?
                    self.add_arp_entry(ip.src, frame.src)?;
                    NetworkProtocol::IP(ip)
                }
                0x0806 => { 
                    // ARP 
                    let arp = ARP::decode(&frame.payload)?;
                    NetworkProtocol::ARP(arp)
                }
                _ => return Err(Error::InvalidBytes { msg: "IP".to_string() })
            };
            if disp {
                print!("{:>3}: ", ctx.t);
                // FIXME: print ip address correct
                println!("{}({}): receive: {:}", 
                         self.base.base.get_name(), 
                         self.ip_addr_ports[0].0, p);
            }

            self.rbuf.push_back(p);
        }
        Ok(self.rbuf.pop_front())
    }

    pub fn push_sbuf(&mut self, p: NetworkProtocol, ctx: &UpdateContext) -> Res<()> {
        match p {
            NetworkProtocol::IP(ip) => self.push_sbuf_ip(ip, ctx)?,
            NetworkProtocol::ARP(arp) => self.push_sbuf_arp(arp, ctx)?,
        };
        Ok(())
    }

    pub fn push_sbuf_ip(&mut self, p: IP, ctx: &UpdateContext) -> Res<()> {
        // FIXME: how to determine src_ip here ?
        let disp = crate::output::is_frame_level();
        if disp {
            print!("{:>3}: ", ctx.t);
            // FIXME: print ip address correct
            println!("{}({}): send   : {:}", 
                     self.base.base.get_name(), 
                     self.ip_addr_ports[0].0, p);
        }

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
        let ethertype = 0x0800;
        let frame = EthernetFrame::new(*dst_mac, src_mac, ethertype, payload);
        self.base.push_sbuf(frame, ctx);

        Ok(())
    }

    pub fn push_sbuf_arp(&mut self, arp: ARP, ctx: &UpdateContext) -> Res<()> {
        let disp = crate::output::is_frame_level();
        if disp {
            print!("{:>3}: ", ctx.t);
            // FIXME: print ip address correct
            println!("{}({}): send   : {:}", 
                     self.base.base.get_name(), 
                     self.ip_addr_ports[0].0, arp);
        }


        let src_mac = self.base.base.get_mac();
        let dst_mac = match arp.opcode {
            1 => Mac::new(999), // request // FIXME: broadcast
            2 => arp.target_mac,// reply
            _ => panic!("invalid arp opcode"),
        };
        let payload = arp.encode();
        let ethertype = 0x0806;
        let frame = EthernetFrame::new(dst_mac, src_mac, ethertype, payload);
        self.base.push_sbuf(frame, ctx);

        Ok(())
    }

    pub fn find_port(&self, nw_part: &NetworkPart) -> Option<Port> {
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

    pub fn add_route_entry(&mut self, nw_part: NetworkPart, ip_addr: IpAddr) -> Res<()> {
        self.routing_table.insert(nw_part, ip_addr);
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

