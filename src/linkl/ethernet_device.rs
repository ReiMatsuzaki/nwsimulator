use std::collections::{VecDeque, HashMap};

use crate::{types::{Mac, Port, Error, UpdateContext}, physl::BaseDevice};

use super::{EthernetFrame, EthernetLog};

pub struct BaseEthernetDevice {
    pub recv_buf: VecDeque<EthernetFrame>,
    pub send_buf: VecDeque<EthernetFrame>,
    forward_table: HashMap<Mac, Port>,
    bufs: HashMap<Port, Vec<u8>>,
    pub base: BaseDevice,

    pub rlog: Vec<EthernetLog>,
    pub slog: Vec<EthernetLog>,
}

impl BaseEthernetDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseEthernetDevice {
        BaseEthernetDevice {
            recv_buf: VecDeque::new(),
            send_buf: VecDeque::new(), // FIXME: sbuf isn't used
            forward_table: HashMap::new(),
            bufs: HashMap::new(),
            base: BaseDevice::new(mac, name, num_ports),
            rlog: Vec::new(),
            slog: Vec::new(),
        }
    }

    pub fn recv(&mut self, ctx: &UpdateContext) -> Option<EthernetFrame> {
        let disp = crate::output::is_frame_level();

        while let Some((port, x)) = self.base.recv() {
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
                        self.recv_buf.push_back(frame);
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

        self.recv_buf.pop_front()
    }

    pub fn send(&mut self, frame: EthernetFrame, ctx: &UpdateContext) {
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
                self.base.send((port, *byte));
            }
        }
    }

    pub fn add_forwarding_table(&mut self, dst: Mac, port: Port) {
        self.forward_table.insert(dst, port);
    }
}

pub trait EthernetDevice {
    fn ether_base(&self) -> &BaseEthernetDevice;

    fn ether_base_mut(&mut self) -> &mut BaseEthernetDevice;

    fn recv(&mut self, ctx: &UpdateContext) -> Option<EthernetFrame> {
        self.ether_base_mut().recv(ctx)
    }

    fn send(&mut self, frame: EthernetFrame, ctx: &UpdateContext) {
        self.ether_base_mut().send(frame, ctx)
    }

    fn add_forwarding_table(&mut self, dst: Mac, port: Port) {
        self.ether_base_mut().add_forwarding_table(dst, port)
    }

    fn get_rlog(&self) -> &Vec<EthernetLog> {
        &self.ether_base().rlog
    }

    fn get_slog(&self) -> &Vec<EthernetLog> {
        &self.ether_base().slog
    }
}
