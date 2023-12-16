use crate::linkl::ethernet_frame::EthernetFrame;

struct BaseEthernetDevice {
    rbuf: VecDeque<EthernetFrame>,
    sbuf: VecDeque<EthernetFrame>,
    base: BaseByteDevice,
}
impl Connectable for BaseEthernetDevice {
    fn get_mac(self) -> Mac {        
        self.base.get_mac()
    }

    fn get_num_ports(self) -> usize {
        self.base.get_num_ports()
    }

    fn receive(&mut self, port: Port, x: u8) {
        self.base.receive(port, x);
        // from self.base.rbuf encode frame and push to self.rbug
        // this execution is for each port
        let mut xs = vec![];
        while let Some((port, x)) = self.base.pop_received() {
            if port == Port::new(0) {
                xs.push(x)
            }
        }
        if let Ok(frame) = EthernetFrame::decode(&mut xs) {
            self.rbuf.push_back(frame);
        }        
    }

    fn send(&mut self, port: Port) -> Option<u8> {
        // from self.sbuf pop frame and decode to self.base.sbuf
        while let Some(frame) = self.sbuf.pop_front() {
            let bytes = EthernetFrame::encode(&frame);
            // FIXME: chose port using forward table
            for byte in bytes {
                self.base.sbuf.push_back((port, byte));
            }
        }
        self.base.send(port)
    }

    fn update(&mut self) {
    }
}

struct Bridge {
    base: BaseEthernetDevice,
}
impl Connectable for Bridge {
    fn get_mac(self) -> Mac {        
        self.base.get_mac()
    }

    fn get_num_ports(self) -> usize {
        self.base.get_num_ports()
    }

    fn receive(&mut self, port: Port, x: u8) {
        self.base.receive(port, x);
    }

    fn send(&mut self, port: Port) -> Option<u8> {
        self.base.send(port)
    }

    fn update(&mut self) {
        while let Some(frame) = self.base.rbuf.pop_front() {
            self.base.sbuf.push_back(frame);
        }
    }
}

