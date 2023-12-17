use std::collections::{VecDeque, HashMap};

use super::types::{Port, Mac};
use super::physl::{BaseDevice, Device, UpdateContext, Network};
use crate::linkl::ethernet_frame::EthernetFrame;


pub struct BaseEthernetDevice {
    pub rbuf: VecDeque<EthernetFrame>,
    pub sbuf: VecDeque<EthernetFrame>,
    forward_table: HashMap<Mac, Port>,
    bufs: HashMap<Port, Vec<u8>>,
    pub base: BaseDevice,
}

impl BaseEthernetDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseEthernetDevice {
        BaseEthernetDevice {
            rbuf: VecDeque::new(),
            sbuf: VecDeque::new(),
            forward_table: HashMap::new(),
            bufs: HashMap::new(),
            base: BaseDevice::new(mac, name, num_ports),
        }
    }

    pub fn pop_received(&mut self) -> Option<EthernetFrame> {
        while let Some((port, x)) = self.base.pop_received() {
            if let Some(xs) = self.bufs.get_mut(&port) {
                xs.push(x);
            } else {
                self.bufs.insert(port, vec![x]);
            }

            if let Some(xs) = self.bufs.get(&port) {
                // FIXME: violated bytes are not consumed
                if let Ok(frame) = EthernetFrame::decode(xs) {
                    self.forward_table.insert(Mac::new(frame.src), port);
                    self.rbuf.push_back(frame);
                }
            }
        }
        self.rbuf.pop_front()
    }

    pub fn push_sending(&mut self, frame: EthernetFrame) {
        let bytes = EthernetFrame::encode(&frame);
        // FIXME: duplicated code
        if let Some(port) = self.forward_table.get(&Mac::new(frame.dst)) {
            for byte in &bytes {
                self.base.push_sending((*port, *byte));
            }
        } else if let Some(src_port) = self.forward_table.get(&Mac::new(frame.src)) {
            for port in 0..self.base.get_num_ports() {
                let port = Port::new(port as u32);
                if port != *src_port {
                    for byte in &bytes {
                        self.base.push_sending((port, *byte));
                    }
                }
            }
        } else {
            for port in 0..self.base.get_num_ports() {
                let port = Port::new(port as u32);
                for byte in &bytes {
                    self.base.push_sending((port, *byte));
                }
            }        
        }
    }

    pub fn handle_frame(&mut self, handler: & dyn Fn(EthernetFrame) -> Vec<EthernetFrame>) {
        while let Some(frame) = self.pop_received() {
            let response_frame_list = handler(frame);
            for f in response_frame_list {
                self.push_sending(f);
            }
        }
    }
}

// // FIXME: delete it?
// impl Device for BaseEthernetDevice {
//     fn base(&self) -> &BaseDevice {
//         &self.base
//     }

//     fn base_mut(&mut self) -> &mut BaseDevice {
//         &mut self.base
//     }

//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }

//     fn update(&mut self, _ctx: &UpdateContext) {}
// }

pub struct EthernetDevice {
    base: BaseEthernetDevice,
    handler: Box<dyn Fn(EthernetFrame) -> Vec<EthernetFrame>>,
    schedules: VecDeque<EithernetLog>,
}

#[derive(Debug)]
pub struct EithernetLog {
    t: usize,
    frame: EthernetFrame,
}

impl EthernetDevice {
    fn new(mac: Mac, name: &str, num_ports: usize, handler: Box<dyn Fn(EthernetFrame) -> Vec<EthernetFrame>>) -> EthernetDevice {
        EthernetDevice {
            base: BaseEthernetDevice::new(mac, name, num_ports),
            handler,
            schedules: VecDeque::new(),
        }
    }

    pub fn build_host(mac: Mac, name: &str) -> Box<EthernetDevice> {
        let handler = Box::new(
            |_frame| vec![]
        );
        Box::new(Self::new(mac, name, 1, handler))
    }

    pub fn build_bridge(mac: Mac, name: &str) -> Box<EthernetDevice> {
        let handler = Box::new(
            |frame| vec![frame]
        );
        Box::new(Self::new(mac, name, 2, handler))
    }

    pub fn add_schedule(&mut self, t: usize, frame: EthernetFrame) {
        self.schedules.push_back(EithernetLog { t, frame });
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

    fn update(&mut self, ctx: &UpdateContext) {
        // println!("update ethernet device: {}", self.get_name());
        if let Some(schedule) = self.schedules.front() {
            // println!("schedule: {:?}", schedule);
            if schedule.t == ctx.t {
                // println!("push scheduleing to sending");
                self.base.push_sending(schedule.frame.clone());
                self.schedules.pop_front();
            }
        }

        self.base.handle_frame(&self.handler)
    }
}

// FIXME: implement EtherDevice which use closure
//        Bridge can be replaced with EithernetDevice
// struct Bridge {
//     base: BaseEthernetDevice,
// }

// impl Bridge {
//     pub fn new_box(mac: Mac, name: &str) -> Box<Bridge> {
//         Box::new(
//             Bridge {
//                 base: BaseEthernetDevice::new(mac, name, 2),
//         })
//     }
// }

// impl Device for Bridge {
//     fn base(&self) -> &BaseDevice {
//         &self.base.base
//     }

//     fn base_mut(&mut self) -> &mut BaseDevice {
//         &mut self.base.base
//     }

//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }

//     fn update(&mut self, _ctx: &UpdateContext) {
//         self.base.handle_frame(&|frame| vec![frame])
//     }
// }

pub fn run_sample() {
    println!("run experimental linkl sample");
    let mac0 = Mac::new(23);
    let mac1 = Mac::new(24);
    let mac2 = Mac::new(25);

    let mut host_a = EthernetDevice::build_host(mac0, "host_a");
    let host_b = EthernetDevice::build_host(mac1, "host_b");
    let brdige = EthernetDevice::build_bridge(mac2, "bridge");

    host_a.add_schedule(0, EthernetFrame::new(mac1.value, mac0.value, 3, vec![11, 12, 13]));

    let mut nw = Network::new(
        vec![host_a, host_b, brdige],
        vec![]
    );
    nw.connect_both(mac0, Port::new(0), mac2, Port::new(0)).unwrap();
    nw.connect_both(mac1, Port::new(0), mac2, Port::new(1)).unwrap();
    nw.run(60).unwrap();

    // let d = nw.get_device(mac0).unwrap();
    // let d = d.as_any().downcast_ref::<Bridge>().unwrap();
    // println!("{}", d.get_name());
}