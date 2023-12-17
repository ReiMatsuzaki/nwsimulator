// use super::linkl::

use std::collections::VecDeque;

use crate::linkl::ethernet_frame::EthernetFrame;

use super::physl::{UpdateContext, Device, Network};
use super::types::{Port, Mac, IpAddr, Res, Error};
use super::linkl::BaseEthernetDevice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IP {
    // FIXME: more fields
    pub src: IpAddr,
    pub dst: IpAddr,
}

impl IP {
    pub fn decode(xs: &[u8]) -> Result<IP, ()> {
        if xs.len() < 8 {
            return Err(());
        }
        let src = (xs[0] as u32) << 24 | (xs[1] as u32) << 16 | (xs[2] as u32) << 8 | (xs[3] as u32);
        let dst = (xs[4] as u32) << 24 | (xs[5] as u32) << 16 | (xs[6] as u32) << 8 | (xs[7] as u32);
        Ok(IP { src: IpAddr::new(src), dst: IpAddr::new(dst) })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut xs = vec![];
        xs.push((self.src.value >> 24) as u8);
        xs.push((self.src.value >> 16) as u8);
        xs.push((self.src.value >> 8) as u8);
        xs.push(self.src.value as u8);
        xs.push((self.dst.value >> 24) as u8);
        xs.push((self.dst.value >> 16) as u8);
        xs.push((self.dst.value >> 8) as u8);
        xs.push(self.dst.value as u8);
        xs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ARP {
    src_mac: Mac,
    src_ipaddr: IpAddr,
    dst_ipaddr: IpAddr,
}

impl ARP {
    pub fn decode(xs: &[u8]) -> Result<ARP, ()> {
        if xs.len() < 8 {
            return Err(());
        }
        let src = (xs[0] as u32) << 24 | (xs[1] as u32) << 16 | (xs[2] as u32) << 8 | (xs[3] as u32);
        let dst = (xs[4] as u32) << 24 | (xs[5] as u32) << 16 | (xs[6] as u32) << 8 | (xs[7] as u32);
        Ok(ARP { 
            src_mac: Mac::new(0),
            src_ipaddr: IpAddr::new(src), 
            dst_ipaddr: IpAddr::new(dst),
         })
    }

    pub fn encode(&self) -> Vec<u8> {    
        // FIXME
        vec![self.src_mac.value as u8,
        self.src_ipaddr.value as u8,
        self.dst_ipaddr.value as u8
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    IP(IP),
    ARP(ARP),
}

impl Protocol {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Protocol::IP(ip) => ip.encode(),
            Protocol::ARP(arp) => arp.encode(),
        }
    }
}

pub struct BaseIpDevice {
    rbuf: VecDeque<Protocol>,
    sbuf: VecDeque<Protocol>,
    pub base: BaseEthernetDevice,
}

impl BaseIpDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseIpDevice {
        BaseIpDevice {
            rbuf: VecDeque::new(),
            sbuf: VecDeque::new(),
            base: BaseEthernetDevice::new(mac, name, num_ports),
        }
    }

    pub fn encode(&mut self) -> Res<()> {
        while let Some(frame) = self.base.rbuf.pop_front() {
            if let Ok(ip) = IP::decode(&frame.payload) {
                self.rbuf.push_back(Protocol::IP(ip));
            } else if let Ok(arp) = ARP::decode(&frame.payload) {
                self.rbuf.push_back(Protocol::ARP(arp));
            } else {
                return Err(Error::DecodeFailed { 
                    payload: frame.payload, 
                    msg: "IP".to_string() });
            }
        }
        Ok(())
    }

    pub fn decode(&mut self) {
        // FIXME
        while let Some(p) = self.sbuf.pop_front() {
            let payload = p.encode();
            let frame = EthernetFrame::new(0, 0, 0, payload);
            self.base.sbuf.push_back(frame);
        }

    }

    pub fn update(&mut self, _ctx: &UpdateContext, 
                  f: &mut dyn FnMut(Protocol) -> Vec<Protocol>) -> Res<()> {
        self.encode()?;
        while let Some(p) = self.rbuf.pop_front() {
            let response_list = f(p);
            for p in response_list {
                self.sbuf.push_back(p)
            }
        }
        self.decode();
        Ok(())
    }
}

pub struct IpHost {
    base: BaseIpDevice,
    schedules: Vec<(usize, Protocol)>,
    receives: Vec<(usize, Protocol)>,
}

impl IpHost {
    pub fn build(mac: Mac, name: &str, num_ports: usize) -> Box<IpHost> {
        let host = IpHost {
            base: BaseIpDevice::new(mac, name, num_ports),
            schedules: vec![],
            receives: vec![],
        };
        Box::new(host)
    }
}

impl Device for IpHost {
    fn base(&self) -> &super::physl::BaseDevice {
        &self.base.base.base()
    }

    fn base_mut(&mut self) -> &mut super::physl::BaseDevice {
        &mut self.base.base.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) {
        for (t, p) in self.schedules.iter() {
            if *t == ctx.t {
                self.base.sbuf.push_back(p.clone());
            }
        }
        let f = &mut |p: Protocol| {
            self.receives.push((ctx.t, p.clone()));
            vec![p]
        };
        self.base.update(ctx, f).unwrap()

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

