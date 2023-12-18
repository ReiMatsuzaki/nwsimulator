use std::collections::{VecDeque, HashMap};

use super::types::{Port, Mac, Res, Error};
use super::utils::{read_6bytes, read_2bytes, split_6bytes, split_2bytes};
use super::physl::{BaseDevice, Device, UpdateContext, Network};

type BytesFn = dyn Fn(&Vec<u8>) -> Res<Option<Vec<u8>>>;

#[derive(Clone, Debug, PartialEq)]
pub struct EthernetFrame {
    pub dst: Mac,       // 6 bytes
    pub src: Mac,       // 6 bytes
    pub ethertype: u16, // 2 bytes
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn new(dst: Mac, src: Mac, ethertype: u16, payload: Vec<u8>) -> EthernetFrame {
        EthernetFrame {
            dst,
            src,
            ethertype,
            payload,
        }
    }

    pub fn decode(xs: &Vec<u8>) -> Res<EthernetFrame> {
        if xs.len() < 8 + 6 + 6 + 2 {
            return Err(Error::NotEnoughBytes);
        }
        for i in 0..7 {
            if xs[i] != 0xAA {
                // 10101010 = 0xAA
                return Err(Error::InvalidBytes {
                    msg: "bad preamble".to_string(),
                });
            }
        }
        if xs[7] != 0xAB {
            // 10101011 = 0xAB
            return Err(Error::InvalidBytes {
                msg: "bad preamble".to_string(),
            });
        }
        let dst = Mac::new(read_6bytes(xs, 8));
        let src = Mac::new(read_6bytes(xs, 8 + 6));
        let ty = read_2bytes(xs, 8 + 6 + 6);
        if ty > 0x05DC {
            return Err(Error::InvalidBytes {
                msg: format!("unsupported ethernet type: {}", ty),
            });
        } else {
            // type is length
            let len = ty as usize;
            if xs.len() < 8 + 6 + 6 + 2 + len {
                return Err(Error::NotEnoughBytes);
            }
            let payload = Vec::from(&xs[(8 + 6 + 6 + 2)..(8 + 6 + 6 + 2 + len)]);
            Ok(EthernetFrame {
                dst,
                src,
                ethertype: ty,
                payload,
            })
        }
    }

    pub fn encode(frame: &EthernetFrame) -> Vec<u8> {
        let et = split_2bytes(frame.ethertype);
        let dst = split_6bytes(frame.dst.value);
        let src = split_6bytes(frame.src.value);
        let mut xs = vec![
            0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAB, // preamble
            dst[0], dst[1], dst[2], dst[3], dst[4], dst[5], src[0], src[1], src[2], src[3], src[4],
            src[5], et[0], et[1],
        ];
        // FIXME: avoid clone
        xs.append(&mut frame.payload.clone());
        xs
    }
}

impl std::fmt::Display for EthernetFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut payload = "[".to_string();
        for x in &self.payload {
            payload = format!("{} {}", payload, x);
        }
        payload = format!("{}]", payload);
        write!(f, "EthernetFrame(dst:{}, src:{}, typ:{}, payload:{})", 
               self.dst.value, self.src.value, self.ethertype, payload)
    }
}

enum DeviceType {
    Host(Box<BytesFn>),
    Hub,
} 

pub struct BaseEthernetDevice {
    pub rbuf: VecDeque<EthernetFrame>,
    pub sbuf: VecDeque<EthernetFrame>,
    forward_table: HashMap<Mac, Port>,
    bufs: HashMap<Port, Vec<u8>>,
    pub base: BaseDevice,

    pub rlog: Vec<EthernetLog>,
    pub slog: Vec<EthernetLog>,
}

impl BaseEthernetDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseEthernetDevice {
        BaseEthernetDevice {
            rbuf: VecDeque::new(),
            sbuf: VecDeque::new(), // FIXME: sbuf isn't used
            forward_table: HashMap::new(),
            bufs: HashMap::new(),
            base: BaseDevice::new(mac, name, num_ports),
            rlog: Vec::new(),
            slog: Vec::new(),
        }
    }

    pub fn pop_rbuf(&mut self, ctx: &UpdateContext) -> Option<EthernetFrame> {
        let disp = crate::output::is_frame_level();

        while let Some((port, x)) = self.base.pop_rbuf() {
            if let Some(xs) = self.bufs.get_mut(&port) {
                xs.push(x);
            } else {
                self.bufs.insert(port, vec![x]);
            }

            if let Some(xs) = self.bufs.get_mut(&port) {
                match EthernetFrame::decode(xs) {
                    Ok(frame) => {
                        if disp {
                            print!("{:>3}: ", ctx.t);
                            println!("{}({}): receive: {:}", self.base.get_name(), self.base.get_mac().value, frame);
                        }
                        self.rlog.push(EthernetLog { t: ctx.t, frame: frame.clone() });
    
                        self.forward_table.insert(frame.src, port);
                        self.rbuf.push_back(frame);
                        xs.clear();
                    },
                    Err(Error::NotEnoughBytes) => {}, // do nothing
                    Err(_) => {
                        println!("{}({}): invalid frame. clear bytes", self.base.get_name(), self.base.get_mac().value);
                        xs.clear(); // clear illegal bytes
                    }
                }
            }
        }

        self.rbuf.pop_front()
    }

    pub fn push_sbuf(&mut self, frame: EthernetFrame, ctx: &UpdateContext) {
        let disp = crate::output::is_frame_level();
        self.slog.push(EthernetLog { t: ctx.t, frame: frame.clone() });
        if disp {
            print!("{:>3}: ", ctx.t);
            println!("{}({}): send:    {:}", self.base.get_name(), self.base.get_mac().value, frame);
        }

        let mut ports = vec![];
        if let Some(port) = self.forward_table.get(&frame.dst) {
            ports.push(*port);
        } else if let Some(src_port) = self.forward_table.get(&frame.src) {
            for port in 0..self.base.get_num_ports() {
                if port != src_port.value as usize {
                    ports.push(Port::new(port as u32));
                }
            }
        } else {
            for port in 0..self.base.get_num_ports() {
                ports.push(Port::new(port as u32));
            }
        };

        let bytes = EthernetFrame::encode(&frame);
        for port in ports {
            for byte in &bytes {
                self.base.push_sbuf((port, *byte));
            }
        }
    }

    pub fn add_forwarding_table(&mut self, dst: Mac, port: Port) {
        self.forward_table.insert(dst, port);
    }
}

pub struct EthernetDevice {
    base: BaseEthernetDevice,
    // handler: Box<BytesFn>,
    device_type: DeviceType,
    schedules: VecDeque<EthernetLog>,
}

impl EthernetDevice {
    fn new(base: BaseEthernetDevice, device_type: DeviceType) -> EthernetDevice {
        EthernetDevice {
            base,
            device_type,
            schedules: VecDeque::new(),
        }
    }

    pub fn build_host(mac: Mac, name: &str) -> Box<EthernetDevice> {
        let base = BaseEthernetDevice::new(mac, name, 1);
        let handler = Box::new(
            |_bytes: &Vec<u8>| Ok(None)
        );
        Box::new(Self::new(base, DeviceType::Host(handler)))
    }

    pub fn build_echo_host(mac: Mac, name: &str) -> Box<EthernetDevice> {
        let base = BaseEthernetDevice::new(mac, name, 1);
        let handler = Box::new(
            |bytes: &Vec<u8>| Ok(Some(bytes.clone()))
        );
        Box::new(Self::new(base, DeviceType::Host(handler)))
    }

    pub fn build_bridge(mac: Mac, name: &str) -> Box<EthernetDevice> {
        let base = BaseEthernetDevice::new(mac, name, 2);
        Box::new(Self::new(base, DeviceType::Hub))
    }

    pub fn build_switch(mac: Mac, name: &str, num_ports: usize) -> Box<EthernetDevice> {
        let base = BaseEthernetDevice::new(mac, name, num_ports);
        Box::new(Self::new(base, DeviceType::Hub))
    }

    pub fn add_schedule(&mut self, t: usize, frame: EthernetFrame) {
        self.schedules.push_back(EthernetLog { t, frame });
    }

    pub fn get_rlog(&self) -> &Vec<EthernetLog> {
        &self.base.rlog
    }

    pub fn get_slog(&self) -> &Vec<EthernetLog> {
        &self.base.rlog
    }
}

impl Device for EthernetDevice {
    fn base(&self) -> &BaseDevice {
        &self.base.base
    }

    fn base_mut(&mut self) -> &mut BaseDevice {
        &mut self.base.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) -> Res<()> {
        if let Some(schedule) = self.schedules.front() {
            if schedule.t == ctx.t {
                self.base.push_sbuf(schedule.frame.clone(), ctx);
                self.schedules.pop_front();
            }
        }

        // rbuf -> sbuf
        while let Some(frame) = self.base.pop_rbuf(ctx) {
            let dst = frame.dst;
            let src = frame.src;
            match &self.device_type {
                DeviceType::Host(f) => {
                    if dst == self.base.base.get_mac() {
                        if let Some(payload) = (f)(&frame.payload)? {
                            let f = EthernetFrame::new(src, dst, payload.len() as u16, payload);
                            self.base.push_sbuf(f, ctx);
                        }
                    } else {
                        // this frame is not for me. just consume it.
                    }
                },
                DeviceType::Hub => {
                    // this frame is not for me. just forward it.
                    self.base.push_sbuf(frame, ctx);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EthernetLog {
    t: usize,
    frame: EthernetFrame,
}

pub fn run_sample() -> Res<EthernetLog> {
    println!("run experimental linkl sample");
    crate::output::set_level(crate::output::Level::Frame);
    let mac0 = Mac::new(23);
    let mac1 = Mac::new(24);
    let mac2 = Mac::new(25);

    let mut host_a = EthernetDevice::build_host(mac0, "host_a");
    let host_b = EthernetDevice::build_host(mac1, "host_b");
    let brdige = EthernetDevice::build_bridge(mac2, "bridge");

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
    let d = d.as_any().downcast_ref::<EthernetDevice>().unwrap();
    println!("{}", d.get_rlog().len());
    println!("{:?}", d.get_slog()[0]);
    Ok(d.get_slog()[0].clone())
}

pub fn run_sample_3host() -> Res<EthernetLog> {
    println!("run experimental linkl 3host sample");
    crate::output::set_level(crate::output::Level::Frame);
    let mac0 = Mac::new(21);
    let mac1 = Mac::new(22);
    let mac2 = Mac::new(23);
    let mac3 = Mac::new(24);
    let mac_s = Mac::new(30);

    let mut host_0 = EthernetDevice::build_host(mac0, "host_a");
    let host_1 = EthernetDevice::build_echo_host(mac1, "host_b");
    let host_2 = EthernetDevice::build_echo_host(mac2, "host_c");
    let host_3 = EthernetDevice::build_host(mac3, "host_d");
    let switch = EthernetDevice::build_switch(mac_s, "switch", 4);

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
    nw.run(150).unwrap();

    let d = nw.get_device(mac0)?;
    let d = d.as_any().downcast_ref::<EthernetDevice>().unwrap();
    println!("{}", d.get_rlog().len());
    let log = &d.get_rlog()[0];
    println!("t={}, frame={}", log.t, log.frame);
    // Ok(d.get_slog()[0].clone())
    Ok(log.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

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