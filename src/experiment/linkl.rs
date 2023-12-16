use std::collections::{VecDeque, HashMap};

use crate::linkl::ethernet_frame::EthernetFrame;

use super::{BaseByteDevice, Connectable, Mac, Port, UpdateContext};

struct BaseEthernetDevice {
    rbuf: VecDeque<EthernetFrame>,
    sbuf: VecDeque<EthernetFrame>,
    forward_table: HashMap<Mac, Port>,
    base: BaseByteDevice,
}

impl BaseEthernetDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseEthernetDevice {
        BaseEthernetDevice {
            rbuf: VecDeque::new(),
            sbuf: VecDeque::new(),
            forward_table: HashMap::new(),
            base: BaseByteDevice::new(mac, name, num_ports),
        }
    }

    pub fn receive_update(&mut self) {
        // FIXME: store xs
        // from self.base.rbuf encode frame and push to self.rbug
        // this execution is for each port
        let mut xs = vec![];
        while let Some((port, x)) = self.base.pop_received() {
            if port == Port::new(0) {
                xs.push(x)
            }
        }
        if let Ok(frame) = EthernetFrame::decode(&mut xs) {
            // FIXME: port is always 0
            let port = Port::new(0);
            self.forward_table.insert(Mac::new(frame.src), port);
            self.rbuf.push_back(frame);
        }        
    }

    pub fn send_update(&mut self) {
        // from self.sbuf pop frame and decode to self.base.sbuf
        while let Some(frame) = self.sbuf.pop_front() {
            let bytes = EthernetFrame::encode(&frame);
            // FIXME: chose port using forward table
            if let Some(port) = self.forward_table.get(&Mac::new(frame.dst)) {
                for byte in bytes {
                    self.base.sbuf.push_back((*port, byte));
                }
            } else {
                panic!("unknown mac address");
            }
        }
    }

    pub fn handle_frame(&mut self, handler: & dyn Fn(EthernetFrame) -> Vec<EthernetFrame>) {
        self.receive_update();
        while let Some(frame) = self.rbuf.pop_front() {
            let response_frame_list = handler(frame);
            for f in response_frame_list {
                self.sbuf.push_back(f)
            }
        }
        self.send_update();
    }
}

impl Connectable for BaseEthernetDevice {
    fn get_mac(&self) -> Mac {        
        self.base.get_mac()
    }

    fn get_num_ports(&self) -> usize {
        self.base.get_num_ports()
    }

    fn get_name(&self) -> &str {
        &self.base.get_name()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn receive(&mut self, port: Port, x: u8) {
        self.base.receive(port, x);
    }

    fn send(&mut self) -> Option<(Port, u8)> {
        self.base.send()
    }

    fn update(&mut self, _ctx: &UpdateContext) {}
}

struct Bridge {
    base: BaseEthernetDevice,
}

impl Bridge {
    pub fn new(mac: Mac, name: &str) -> Box<Bridge> {
        Box::new(
            Bridge {
                base: BaseEthernetDevice::new(mac, name, 2),
        })
    }
}

impl Connectable for Bridge {
    fn get_mac(&self) -> Mac {        
        self.base.get_mac()
    }

    fn get_num_ports(&self) -> usize {
        self.base.get_num_ports()
    }

    fn get_name(&self) -> &str {
        &self.base.get_name()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn receive(&mut self, port: Port, x: u8) {
        self.base.receive(port, x);
    }

    fn send(&mut self) -> Option<(Port, u8)> {
        self.base.send()
    }

    fn update(&mut self, _ctx: &UpdateContext) {
        self.base.handle_frame(&|frame| vec![frame])
        // self.base.receive_update();
        // while let Some(frame) = self.base.rbuf.pop_front() {

        //     let response_frame = frame;

        //     self.base.sbuf.push_back(response_frame)
        // }
        // self.base.send_update();
    }
}

pub fn run_sample() {
    println!("run experimental linkl sample");
    let mac0 = Mac::new(23);
    let mac1 = Mac::new(24);

    let bridge0 = Bridge::new(mac0, "bridge0");
    let bridge1 = Bridge::new(mac1, "bridge1");

    let mut nw = super::Network::new(
        vec![bridge0, bridge1],
        vec![]
    );
    nw.connect_both(mac0, Port::new(0), mac1, Port::new(0)).unwrap();
    nw.run(10).unwrap();

    let d = nw.get_device(mac0).unwrap();
    let d = d.as_any().downcast_ref::<Bridge>().unwrap();
    println!("{}", d.get_name());
}