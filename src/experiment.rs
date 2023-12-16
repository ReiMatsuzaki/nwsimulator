use std::collections::VecDeque;

pub mod linkl;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mac(u64);

impl Mac {
    fn new(value: u64) -> Mac {
        Mac(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(u32);

impl Port {
    fn new(value: u32) -> Port {
        Port(value)
    }
}

type PortByteQue = VecDeque<(Port, u8)>;

pub struct Media {
    pub mac0: Mac,
    pub port0: Port,
    pub mac1: Mac,
    pub port1: Port,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateContext {
    t: usize,
}
use std::any::Any;
pub trait Connectable {
    fn get_mac(&self) -> Mac;
    fn get_name(&self) -> &str;
    fn get_num_ports(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn receive(&mut self, port: Port, x: u8);
    fn send(&mut self) -> Option<(Port, u8)>;
    fn update(&mut self, _ctx: &UpdateContext);
}

pub struct BaseByteDevice {
    mac: Mac,
    name: String,
    num_ports: usize,
    rbuf: PortByteQue,
    sbuf: PortByteQue,
}
impl BaseByteDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseByteDevice {
        BaseByteDevice {
            mac,
            name: name.to_string(),
            num_ports,
            rbuf: PortByteQue::new(),
            sbuf: PortByteQue::new(),
        }
    }

    pub fn pop_received(&mut self) -> Option<(Port, u8)> {
        self.rbuf.pop_front()
    }

    pub fn push_sending(&mut self, x: (Port, u8)) {
        self.sbuf.push_back(x)
    }
}
impl Connectable for BaseByteDevice {
    fn get_mac(&self) -> Mac {
        self.mac
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_num_ports(&self) -> usize {
        self.num_ports
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn receive(&mut self, port: Port, x: u8) {
        self.rbuf.push_back((port, x));
    }

    fn send(&mut self) -> Option<(Port, u8)> {
        self.sbuf.pop_front()
    }

    fn update(&mut self, _ctx: &UpdateContext) {
        // do nothing
    }
}

struct Repeater {
    base: BaseByteDevice,
}
impl Repeater {
    pub fn new(mac: Mac, name: &str) -> Repeater {
        Repeater {
            base: BaseByteDevice::new(mac, name, 2),
        }
    }
}
impl Connectable for Repeater {
    fn get_mac(&self) -> Mac {
        self.base.get_mac()
    }

    fn get_name(&self) -> &str {
        self.base.get_name()
    }

    fn get_num_ports(&self) -> usize {
        self.base.get_num_ports()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn send(&mut self) -> Option<(Port, u8)> {
        self.base.send()
    }

    fn receive(&mut self, port: Port, x: u8) {
        self.base.receive(port, x);
    }

    fn update(&mut self, _ctx: &UpdateContext) {
        while let Some((p, x)) = self.base.pop_received() {
            self.base.push_sending((Port::new(1-p.0), x));
        }
    }
}

struct ByteHost {
    base: BaseByteDevice,
    schedules: Vec<(usize, Port, u8)>,
    receives: Vec<(usize, Port, u8)>,
}
impl ByteHost {
    pub fn new(mac: Mac, name: &str, schedules: Vec<(usize, Port, u8)>) -> ByteHost {
        ByteHost {
            base: BaseByteDevice::new(mac, name, 1),
            schedules,
            receives: Vec::new(),
        }
    }
}
impl Connectable for ByteHost {
    fn get_mac(&self) -> Mac {
        self.base.get_mac()
    }

    fn get_name(&self) -> &str {
        self.base.get_name()
    }

    fn get_num_ports(&self) -> usize {
        self.base.get_num_ports()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn send(&mut self) -> Option<(Port, u8)> {
        self.base.send()
    }

    fn receive(&mut self, port: Port, x: u8) {
        self.base.receive(port, x);
    }

    fn update(&mut self, ctx: &UpdateContext) {
        while let Some(x) = self.base.pop_received() {
            self.receives.push((ctx.t, x.0, x.1));
        }

        for (t, port, x) in &self.schedules {
            if *t == ctx.t {
                self.base.push_sending((*port, *x));
            }
        }
    }
}

trait Device {
    fn get_num(&self) -> u8;
}

struct Network {
    devices: Vec<Box<dyn Connectable>>,
    medias: Vec<Media>,
}
type Res<T> = Result<T, String>;
impl Network {
    fn new(devices: Vec<Box<dyn Connectable>>, medias: Vec<Media>) -> Network {
        Network { devices, medias }
    }

    fn connect(&mut self, mac0: Mac, port0: Port, mac1: Mac, port1: Port) -> Res<()> {
        self.medias.push(Media {
            mac0,
            port0,
            mac1,
            port1,
        });
        Ok(())
    }

    pub fn connect_both(&mut self, mac0: Mac, port0: Port, mac1: Mac, port1: Port) -> Res<()> {
        self.connect(mac0, port0, mac1, port1)?;
        self.connect(mac1, port1, mac0, port0)?;
        Ok(())
    }

    pub fn get_device(&mut self, mac: Mac) -> Res<&mut Box<dyn Connectable>> {
        self.devices
            .iter_mut()
            .find(|d| d.get_mac() == mac)
            .ok_or(format!("device not found: {:?}", mac))
    }

    fn step(&mut self, t: usize) -> Res<()> {
        for idx in 0..self.devices.len() {
            let d = &mut self.devices[idx];
            if let Some((src_port, x)) = d.send() {
                let src_mac = d.get_mac();
                println!("send from {:?}:{:?} x {:?}", d.get_mac(), src_port, x);

                if let Some(m) = self.medias.iter().find(|m| 
                    m.mac0 == src_mac &&
                    m.port0 == src_port
                ) {
                    println!("receive to {:?}:{:?}", m.mac1, m.port1);
                    let mac1 = m.mac1;
                    let port1 = m.port1;
                    self.get_device(mac1)?
                    .receive(port1, x);
                } else {
                    println!("media not found");
                }
            }
        }

        for d in &mut self.devices {
            let ctx = UpdateContext { t };
            d.update(&ctx);
        }
        Ok(())
    }

    pub fn run(&mut self, maxt: usize) -> Res<()> {
        for t in 0..maxt {
            self.step(t)?;
        }
        Ok(())
    }
}

pub fn run_sample() {
    println!("experimental sample run");
    let mac0 = Mac::new(23);
    let repeater = Box::new(Repeater::new(mac0, "repeater0"));
    let mac1 = Mac::new(24);
    let schedules = vec![
        (0, Port::new(0), 0x01),
        (1, Port::new(0), 0x02),
        (2, Port::new(0), 0x03),
        (3, Port::new(0), 0x04),
    ];
    let host1 = Box::new(ByteHost::new(mac1, "host1", schedules));
    let mac2 = Mac::new(25);
    let host2 = Box::new(ByteHost::new(mac2, "host2", vec![]));

    let mut nw = Network::new(
        vec![repeater, host1, host2],
        vec![]
    );
    nw.connect_both(mac0, Port::new(0), mac1, Port::new(0)).unwrap();
    nw.connect_both(mac0, Port::new(1), mac2, Port::new(0)).unwrap();
    nw.run(10).unwrap();

    let d = nw.get_device(mac2).unwrap();
    let d = d.as_any().downcast_ref::<ByteHost>().unwrap();
    println!("{:?}", d.receives);
}
