use std::collections::HashMap;
use super::super::physl::Device;

use super::super::types::*;
use super::super::linkl::{BaseEthernetDevice, EthernetFrame, MAC_BROADCAST};
use super::{network_protocol::*, NetworkLog};
use super::ip::*;
use super::arp::*;
use super::ip_addr::*;

pub struct BaseIpDevice {
    pub base: BaseEthernetDevice,

    // rbuf: VecDeque<NetworkProtocol>,
    // sbuf: VecDeque<Protocol>,
    pub subnet_mask: SubnetMask,
    pub ip_addr_ports: Vec<(IpAddr, Port)>,
    pub routing_table: HashMap<NetworkPart, IpAddr>,
    pub arp_table: HashMap<IpAddr, Mac>,

    slog: Vec<NetworkLog>,
    rlog: Vec<NetworkLog>,
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
        let base = BaseEthernetDevice::new(mac, name, ip_addr_ports.len());
        let device = BaseIpDevice {
            base,
            subnet_mask,
            ip_addr_ports: ip_addr_ports,
            routing_table: HashMap::new(),
            arp_table: HashMap::new(),
            slog: Vec::new(),
            rlog: Vec::new(),
        };
        device
    }

    fn recv_frame(&mut self, ctx: &UpdateContext) -> Option<EthernetFrame> {
        self.base.recv(ctx)
    }

    fn send_frame(&mut self, frame: EthernetFrame, ctx: &UpdateContext) {
        self.base.send(frame, ctx);
    }

    fn find_next_mac(&self, ip_addr: IpAddr) -> Res<Mac> {
        // FIXME: default routing
        let nw_part = NetworkPart::new(ip_addr, self.subnet_mask);
        if let Some(_port) = self.find_port(&nw_part) {
            // dst is in same network
            if let Some(mac) = self.arp_table.get(&ip_addr) {
                Ok(*mac)
            } else {
                panic!("failed to find in arp table. ip.dst={}", ip_addr);
            }
        } else if let Some(next_ip_addr) = self.routing_table.get(&nw_part) {
            if let Some(dst_mac) = self.arp_table.get(next_ip_addr) {
                Ok(*dst_mac)
            } else {
                panic!("failed to find in arp table");
            }
        } else {
            Err(Error::MacNotFailed)
        }
    }

    fn is_for_me(&self, ip_addr: &IpAddr) -> bool {
        self.ip_addr_ports
        .iter()
        .any(|(ip, _)| *ip == *ip_addr)
    }

    fn find_port(&self, nw_part: &NetworkPart) -> Option<Port> {
        self.ip_addr_ports
        .iter()
        .find(|(ip_addr, _)| {
            NetworkPart::new(*ip_addr, self.subnet_mask) == *nw_part
        })
        .map(|(_, port)| *port)
    }

    pub fn get_ip_addr(&self, port: Port) -> Option<IpAddr> {
        self.ip_addr_ports
        .iter()
        .find(|(_, p)| *p == port)
        .map(|(ip_addr, _)| *ip_addr)
    }

    pub fn get_arp_table(&self) -> &HashMap<IpAddr, Mac> {
        &self.arp_table
    }

    pub fn add_arp_entry(&mut self, ip_addr: IpAddr, mac: Mac) -> Res<()> {
        self.arp_table.insert(ip_addr, mac);
        Ok(())
    }

    pub fn add_route_entry(&mut self, nw_part: NetworkPart, ip_addr: IpAddr) -> Res<()> {
        self.routing_table.insert(nw_part, ip_addr);
        Ok(())
    }

    fn decode(&self, frame: &EthernetFrame) -> Res<Option<NetworkProtocol>> {
        if frame.dst != self.get_mac() && !frame.is_bloadcast() { 
            return Ok(None)
        }
        let p = match frame.ethertype {
            0x0800 => { 
                // IPv4
                let ip = IP::decode(&frame.payload)?;
                NetworkProtocol::IP(ip)
            }
            0x0806 => { 
                // ARP 
                let arp = ARP::decode(&frame.payload)?;
                NetworkProtocol::ARP(arp)
            }
            _ => return Err(Error::InvalidBytes { msg: "IP".to_string() })
        };
        Ok(Some(p) )
    }

    fn encode(&self, p: &NetworkProtocol) -> Res<EthernetFrame> {
        let src_mac = self.base().get_mac();
        let dst_mac = match p {
            NetworkProtocol::IP(ip) => {
                self.find_next_mac(ip.dst)?
            }
            NetworkProtocol::ARP(arp) => {
                if arp.opcode == 1 {
                    // request
                    MAC_BROADCAST
                } else {
                    // response
                    arp.target_mac
                }
            }
        };

        let payload = p.encode();
        let ethertype = match p {
            NetworkProtocol::IP(_) => 0x0800,
            NetworkProtocol::ARP(_) => 0x0806,
        };
        let frame = EthernetFrame::new(dst_mac, src_mac, ethertype, payload);
        Ok(frame)
    }

    fn handle_arp(&mut self, arp: &ARP) -> Res<Option<NetworkProtocol>> {
        if self.is_for_me(&arp.target_ipaddr) {
            match arp.opcode {
                1 => { // request
                    let arp = arp.reply(self.get_mac());
                    let arp = NetworkProtocol::ARP(arp);
                    Ok(Some(arp))
                }
                2 => { // reply
                    self.add_arp_entry(arp.sender_ipaddr, arp.sender_mac)?;
                    Ok(None)
                }
                _ => panic!("invalid arp opcode"),
            }
        } else {
            match arp.opcode {
                1 => Ok(None),
                2 => Ok(None),
                _ => panic!("invalid arp opcode"),
            }
        }
    }

    fn add_slog(&mut self, p: &NetworkProtocol, ctx: &UpdateContext) {
        let disp = crate::output::is_frame_level();
        if disp {
            print!("{:>3}: ", ctx.t);
            // FIXME: print ip address correct
            println!("{}({}): send   : {:}", 
                     self.get_name(), 
                     self.ip_addr_ports[0].0, p);
        }

        let log = NetworkLog { t:ctx.t, p: p.clone()} ;
        self.slog.push(log);
    }

    fn add_rlog(&mut self, p: &NetworkProtocol, ctx: &UpdateContext) {
        let disp = crate::output::is_frame_level();
        if disp {
            print!("{:>3}: ", ctx.t);
            // FIXME: print ip address correct
            println!("{}({}): receive: {:}",             
                     self.get_name(), 
                     self.ip_addr_ports[0].0, p);
        }

        let log = NetworkLog { t:ctx.t, p: p.clone()} ;
        self.rlog.push(log);
    }

    pub fn get_rlog(&self) -> &Vec<NetworkLog> {
        &self.rlog
    }

    fn update_table(&mut self) -> Res<()> {
        for (ip_addr, mac) in &self.arp_table {
            let nw_part = NetworkPart::new(*ip_addr, self.subnet_mask);
            if let Some(port) = self.find_port(&nw_part) {
                self.base.add_forwarding_table(*mac, port);
            }
        }
        Ok(())
    }

    fn unreachable(&mut self, p: NetworkProtocol) -> NetworkProtocol {
        let dst = match p {
            NetworkProtocol::IP(ip) => ip.src,
            _ => panic!("unreachable for ARP is not supported"),
        };
        let src = self.ip_addr_ports[0].0;
        let ip = IP::new_icmp(src, dst, 3, 1);
        NetworkProtocol::IP(ip)
    }

    pub fn recv(&mut self, ctx: &UpdateContext) -> Res<Option<NetworkProtocol>> {
        if let Some(frame) = self.recv_frame(ctx) {
            if let Some(p) = self.decode(&frame)? {        
                self.add_rlog(&p, ctx);
                return Ok(Some(p))
            }
        }
        Ok(None)
    }

    pub fn send(&mut self, p: NetworkProtocol, ctx: &UpdateContext)  -> Res<()> {
        match self.encode(&p) {
            Ok(frame) => {
                self.add_slog(&p, ctx);
                self.send_frame(frame, ctx);
                Ok(())
            },
            Err(Error::MacNotFailed) => {
                let ip = self.unreachable(p);
                self.send(ip, ctx)
            },
            Err(e) => return Err(e)
        }
    }
}

impl Device for BaseIpDevice {
    fn base(&self) -> &super::super::physl::BaseDevice {
        &self.base.base
    }

    fn base_mut(&mut self) -> &mut super::super::physl::BaseDevice {
        &mut self.base.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, _ctx: &UpdateContext) -> Res<()> {
        panic!("not implemented")
    }
}

pub trait IpDevice {
    fn ip_base(&self) -> &BaseIpDevice;

    fn ip_base_mut(&mut self) -> &mut BaseIpDevice;

    fn send(&mut self, p: NetworkProtocol, ctx: &UpdateContext) -> Res<()> {
        self.ip_base_mut().send(p, ctx)
    }

    fn recv(&mut self, ctx: &UpdateContext) -> Res<Option<NetworkProtocol>> {
        self.ip_base_mut().recv(ctx)
    }

    fn base_handle_arp(&mut self, arp: &ARP) -> Res<Option<NetworkProtocol>> {
        self.ip_base_mut().handle_arp(arp)
    }

    fn add_arp_entry(&mut self, ip_addr: IpAddr, mac: Mac) -> Res<()> {
        self.ip_base_mut().add_arp_entry(ip_addr, mac)
    }

    fn add_route_entry(&mut self, nw_part: NetworkPart, ip_addr: IpAddr) -> Res<()> {
        self.ip_base_mut().add_route_entry(nw_part, ip_addr)
    }

    fn get_ip_addr(&self, port: Port) -> Option<IpAddr> {
        self.ip_base().get_ip_addr(port)
    }

    fn get_arp_table(&self) -> &HashMap<IpAddr, Mac> {
        self.ip_base().get_arp_table()
    }

    fn get_rlog(&self) -> &Vec<NetworkLog> {
        self.ip_base().get_rlog()
    }

    fn handle(&mut self, p: &NetworkProtocol, ctx: &UpdateContext) -> Res<Option<NetworkProtocol>> {
        match p {
            NetworkProtocol::IP(ip) => self.handle_ip(ip, ctx),
            NetworkProtocol::ARP(arp) => self.ip_base_mut().handle_arp(arp),
        }
    }

    fn handle_ip(&mut self, ip: &IP, ctx: &UpdateContext) -> Res<Option<NetworkProtocol>> {
        if self.ip_base().is_for_me(&ip.dst) {
            match &ip.payload {
                IpPayload::ICMP { ty: 3, code } => { // unreachable
                    Err(Error::IpUnreashcable { 
                        code: *code,
                        msg: "".to_string() 
                    })
                }
                IpPayload::ICMP { ty, code} => {
                    panic!("unimplemented ICMP ty={} code={}", ty, code)
                }
                IpPayload::Bytes(xs) => {
                    if let Some(payload) = self.handle_ip_reply(&xs, ctx)? {
                        let ip = IP::new_byte(ip.dst, ip.src, payload);
                        let ip = NetworkProtocol::IP(ip);
                        Ok(Some(ip))
                    } else {
                        Ok(None)
                    }

                }
            }
        } else {
            match &ip.payload {
                IpPayload::ICMP { ty: _, code: _ } => Ok(None),
                IpPayload::Bytes(_) => {
                    let p = NetworkProtocol::IP(ip.clone());
                    Ok(Some(p))
                }
            }
        }
    }

    fn handle_ip_reply(&mut self, bytes: &Vec<u8>, ctx: &UpdateContext) -> Res<Option<Vec<u8>>>;

    fn base_update(&mut self, ctx: &UpdateContext) -> Res<()> {
        while let Some(p) = self.recv(ctx)? {
            if let Some(p) = self.handle(&p, ctx)? {
                self.send(p, ctx)?;
            }
        }
        self.ip_base_mut().update_table()?;
        Ok(())
    }
}