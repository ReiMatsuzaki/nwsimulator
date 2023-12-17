use std::collections::VecDeque;
use std::any::Any;
use super::types::*;

pub struct Media {
    pub mac0: Mac,
    pub port0: Port,
    pub mac1: Mac,
    pub port1: Port,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateContext {
    pub t: usize,
}

pub trait Device {
    fn base(&self) -> &BaseByteDevice;

    fn base_mut(&mut self) -> &mut BaseByteDevice;

    fn get_mac(&self) -> Mac {
        self.base().mac
    }

    fn get_name(&self) -> &str {
        &self.base().name
    }

    fn get_num_ports(&self) -> usize {
        self.base().num_ports
    }

    fn receive(&mut self, port: Port, x: u8) {
        self.base_mut().rbuf.push_back((port, x));
    }

    fn send(&mut self) -> Option<(Port, u8)> {
        self.base_mut().sbuf.pop_front()
    }

    fn as_any(&self) -> &dyn Any;

    fn update(&mut self, _ctx: &UpdateContext);
}

pub struct BaseByteDevice {
    mac: Mac,
    name: String,
    num_ports: usize,
    rbuf: VecDeque<(Port, u8)>,
    sbuf: VecDeque<(Port, u8)>,
}

impl BaseByteDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseByteDevice {
        BaseByteDevice {
            mac,
            name: name.to_string(),
            num_ports,
            rbuf: VecDeque::new(),
            sbuf: VecDeque::new(),
        }
    }

    pub fn pop_received(&mut self) -> Option<(Port, u8)> {
        self.rbuf.pop_front()
    }

    pub fn push_sending(&mut self, x: (Port, u8)) {
        self.sbuf.push_back(x)
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
impl Device for Repeater {
    fn base(&self) -> &BaseByteDevice {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseByteDevice {
        &mut self.base
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, _ctx: &UpdateContext) {
        while let Some((p, x)) = self.base.pop_received() {
            self.base.push_sending((Port::new(1-p.value()), x));
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ByteLog {
    t: usize,
    port: Port,
    x: u8,
}

struct ByteHost {
    base: BaseByteDevice,
    schedules: Vec<ByteLog>,
    receives: Vec<ByteLog>,
}
impl ByteHost {
    pub fn new(mac: Mac, name: &str, schedules: Vec<ByteLog>) -> ByteHost {
        ByteHost {
            base: BaseByteDevice::new(mac, name, 1),
            schedules,
            receives: Vec::new(),
        }
    }
}
impl Device for ByteHost {
    fn base(&self) -> &BaseByteDevice {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseByteDevice {
        &mut self.base
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) {
        while let Some((port, x)) = self.base.pop_received() {
            self.receives.push(ByteLog { t: ctx.t, port, x });
        }
        for ByteLog {t, port, x} in &self.schedules {
            if *t == ctx.t {
                self.base.push_sending((*port, *x));
            }
        }
    }
}

pub struct Network {
    devices: Vec<Box<dyn Device>>,
    medias: Vec<Media>,
}

impl Network {
    pub fn new(devices: Vec<Box<dyn Device>>, medias: Vec<Media>) -> Network {
        Network { devices, medias }
    }

    fn connect(&mut self, mac0: Mac, port0: Port, mac1: Mac, port1: Port) -> Res<()> {
        for (m, p) in [(mac0, port0), (mac1, port1)] {
            let d = self.get_device(m)?;
            if p.value() >= d.get_num_ports().try_into().unwrap() {
                return Err(Error::InvalidPort { mac: mac0, name: d.get_name().to_string(), port: p });
            }
        }

        if mac0 == mac1 {
            return Err(Error::NetworkConnectFailed { mac0, mac1, msg: "same mac address".to_string() })
        }

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

    pub fn get_device(&mut self, mac: Mac) -> Res<&mut Box<dyn Device>> {
        self.devices
            .iter_mut()
            .find(|d| d.get_mac() == mac)
            .ok_or(Error::DeviceNotFound { mac })
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

pub fn run_sample() -> Res<Vec<ByteLog>> {
    println!("experimental sample run");
    let mac0 = Mac::new(23);
    let repeater = Box::new(Repeater::new(mac0, "repeater0"));
    let mac1 = Mac::new(24);
    let schedules = vec![
        ByteLog{ t: 0, port: Port::new(0), x: 0x01 },
        ByteLog{ t: 1, port: Port::new(0), x: 0x02 },
        ByteLog{ t: 2, port: Port::new(0), x: 0x03 },
        ByteLog{ t: 3, port: Port::new(0), x: 0x04 },
    ];
    let host1 = Box::new(ByteHost::new(mac1, "host1", schedules));
    let mac2 = Mac::new(25);
    let host2 = Box::new(ByteHost::new(mac2, "host2", vec![]));

    let mut nw = Network::new(
        vec![repeater, host1, host2],
        vec![]
    );
    nw.connect_both(mac0, Port::new(0), mac1, Port::new(0))?;
    nw.connect_both(mac0, Port::new(1), mac2, Port::new(0))?;
    nw.run(10)?;

    let d = nw.get_device(mac2)?;
    let d = d.as_any().downcast_ref::<ByteHost>().unwrap();
    println!("{:?}", d.receives);
    Ok(d.receives.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2byte_host() {
        let log = run_sample().unwrap();
        assert_eq!(log[0], ByteLog {t: 2, port: Port::new(0), x: 0x01});
        assert_eq!(log[2], ByteLog {t: 4, port: Port::new(0), x: 0x03});
    }
}