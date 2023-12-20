use std::collections::{VecDeque, HashMap};
use crate::experiment::physl::Device;

use super::super::types::*;
use super::super::physl::UpdateContext;
use super::super::linkl::{BaseEthernetDevice, EthernetFrame, MAC_BROADCAST};
use super::{network_protocol::*, NetworkLog};
use super::ip::*;
use super::arp::*;
use super::ip_addr::*;

/*
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

    fn pop_frame(&mut self, ctx: &UpdateContext) -> Option<EthernetFrame> {
        self.base.pop_rbuf(ctx)
    }

    fn push_frame(&mut self, frame: EthernetFrame, ctx: &UpdateContext) {
        self.base.push_sbuf(frame, ctx);
    }

    fn find_mac(&self, ip_addr: IpAddr) -> Res<Mac> {
        let dst_nw_part = NetworkPart::new(ip_addr, self.subnet_mask);
        let dst_mac = if let Some(port) = self.find_port(&dst_nw_part) {
            // dst is in same network
            if let Some(dst_mac) = self.arp_table.get(&ip_addr) {
                dst_mac
            } else {
                panic!("failed to find in arp table. ip.dst={}", ip_addr);
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
        Ok(dst_mac.clone())
    }

}
*/

type BytesFn = dyn Fn(&Vec<u8>) -> Res<Vec<u8>>;

pub struct IpDevice {
    pub base: BaseEthernetDevice,

    // rbuf: VecDeque<NetworkProtocol>,
    // sbuf: VecDeque<Protocol>,
    pub subnet_mask: SubnetMask,
    pub ip_addr_ports: Vec<(IpAddr, Port)>,
    pub routing_table: HashMap<NetworkPart, IpAddr>,
    pub arp_table: HashMap<IpAddr, Mac>,

    schedules: Vec<NetworkLog>,
    slog: Vec<NetworkLog>,
    rlog: Vec<NetworkLog>,
    ip_handler: Box<BytesFn>,
}

impl IpDevice {
    pub fn new(mac: Mac, name: &str, ip_addr_list: Vec<IpAddr>, subnet_mask: SubnetMask, ip_handler: Box<BytesFn>) -> Box<IpDevice> {
        let ip_addr_ports: Vec<(IpAddr, Port)> = ip_addr_list
            .into_iter()
            .enumerate()
            .map(|(i, ip_addr)| {
                (ip_addr, Port::new(i as u32))
            })
            .collect();
        // let num_ports = ip_addr_ports.len();
        let base = BaseEthernetDevice::new(mac, name, ip_addr_ports.len());
        let device = IpDevice {
            base,
            subnet_mask,
            ip_addr_ports: ip_addr_ports,
            routing_table: HashMap::new(),
            arp_table: HashMap::new(),
            schedules: Vec::new(),
            slog: Vec::new(),
            rlog: Vec::new(),
            ip_handler,
        };
        Box::new(device)
    }

    pub fn new_host(mac: Mac, name: &str, ip_addr: IpAddr, subnet_mask: SubnetMask, ip_handler: Box<BytesFn>) -> Box<IpDevice> {
        IpDevice::new(mac, name, vec![ip_addr], subnet_mask, ip_handler)
    }            

    pub fn new_host_echo(mac: Mac, name: &str, ip_addr: IpAddr, subnet_mask: SubnetMask) -> Box<IpDevice> {
        let ip_handler = Box::new(|xs: &Vec<u8>| Ok(xs.clone()) );
        IpDevice::new_host(mac, name, ip_addr, subnet_mask, ip_handler)
    }

    pub fn new_router(mac: Mac, name: &str, ip_addr_list: Vec<IpAddr>, subnet_mask: SubnetMask) -> Box<IpDevice> {
        let ip_handler = Box::new(|_: &Vec<u8>| Ok("i am router".bytes().collect()));
        IpDevice::new(mac, name, ip_addr_list, subnet_mask, ip_handler)
    }

    fn pop_frame(&mut self, ctx: &UpdateContext) -> Option<EthernetFrame> {
        self.base.pop_rbuf(ctx)
    }

    fn push_frame(&mut self, frame: EthernetFrame, ctx: &UpdateContext) {
        self.base.push_sbuf(frame, ctx);
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

    pub fn add_schedule(&mut self, t: usize, p: NetworkProtocol) {
        self.schedules.push(NetworkLog { t, p } );
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

    fn handle(&mut self, p: &NetworkProtocol) -> Res<Option<NetworkProtocol>> {
        match p {
            NetworkProtocol::IP(ip) => self.handle_ip(ip),
            NetworkProtocol::ARP(arp) => self.handle_arp(arp),
        }
    }

    fn handle_ip(&self, ip: &IP) -> Res<Option<NetworkProtocol>> {
        if self.is_for_me(&ip.dst) {
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
                    let payload = (self.ip_handler)(&xs)?;
                    let ip = IP::new_byte(ip.dst, ip.src, payload);
                    let ip = NetworkProtocol::IP(ip);
                    Ok(Some(ip))
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
}

impl Device for IpDevice {
    fn base(&self) -> &super::super::physl::BaseDevice {
        &self.base.base
    }

    fn base_mut(&mut self) -> &mut super::super::physl::BaseDevice {
        &mut self.base.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) -> Res<()> {
        // FIXME: rbuf should be member field?
        let mut rbuf = VecDeque::new();
        let mut sbuf = VecDeque::new();

        for s in &self.schedules {
            if s.t == ctx.t {
                sbuf.push_back(s.p.clone());
            }
        }

        while let Some(frame) = self.pop_frame(ctx) {
            if let Some(p) = self.decode(&frame)? {
                self.add_rlog(&p, ctx);
                rbuf.push_back(p);
            }
        }

        while let Some(p) = rbuf.pop_front() {
            if let Some(p) = self.handle(&p)? {
                sbuf.push_back(p);
            }
        }

        while let Some(p) = sbuf.pop_front() {
            let frame = match self.encode(&p) {
                Ok(frame) => frame,
                Err(Error::MacNotFailed) => {
                    let ip = self.unreachable(p);
                    sbuf.push_back(ip);
                    continue;
                },
                Err(e) => return Err(e)
            };
            self.add_slog(&p, ctx);
            self.push_frame(frame, ctx);
        }

        self.update_table()?;

        Ok(())
    }
}
