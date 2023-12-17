use super::types::*;
use std::any::Any;
use std::collections::VecDeque;

pub struct Connection {
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
    fn base(&self) -> &BaseDevice;

    fn base_mut(&mut self) -> &mut BaseDevice;

    fn get_mac(&self) -> Mac {
        self.base().mac
    }

    fn get_name(&self) -> &str {
        &self.base().name
    }

    fn get_num_ports(&self) -> usize {
        self.base().num_ports
    }

    fn push_rbuf(&mut self, port: Port, x: u8) {
        self.base_mut().rbuf.push_back((port, x));
    }

    fn pop_sbuf(&mut self) -> Option<(Port, u8)> {
        self.base_mut().sbuf.pop_front()
    }

    fn as_any(&self) -> &dyn Any;

    fn update(&mut self, _ctx: &UpdateContext) -> Res<()>;
}

pub struct BaseDevice {
    mac: Mac,
    name: String,
    num_ports: usize,
    rbuf: VecDeque<(Port, u8)>,
    sbuf: VecDeque<(Port, u8)>,
}

impl BaseDevice {
    pub fn new(mac: Mac, name: &str, num_ports: usize) -> BaseDevice {
        BaseDevice {
            mac,
            name: name.to_string(),
            num_ports,
            rbuf: VecDeque::new(),
            sbuf: VecDeque::new(),
        }
    }

    pub fn get_mac(&self) -> Mac {
        self.mac
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_num_ports(&self) -> usize {
        self.num_ports
    }

    pub fn pop_rbuf(&mut self) -> Option<(Port, u8)> {
        self.rbuf.pop_front()
    }

    pub fn push_sbuf(&mut self, x: (Port, u8)) {
        self.sbuf.push_back(x)
    }
}

struct Repeater {
    base: BaseDevice,
}

impl Repeater {
    pub fn new(mac: Mac, name: &str) -> Repeater {
        Repeater {
            base: BaseDevice::new(mac, name, 2),
        }
    }
}

impl Device for Repeater {
    fn base(&self) -> &BaseDevice {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseDevice {
        &mut self.base
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, _ctx: &UpdateContext) -> Res<()> {
        while let Some((p, x)) = self.base.pop_rbuf() {
            self.base.push_sbuf((Port::new(1 - p.value), x));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ByteLog {
    t: usize,
    port: Port,
    x: u8,
}

struct ByteHost {
    base: BaseDevice,
    schedules: Vec<ByteLog>,
    receives: Vec<ByteLog>,
}

impl ByteHost {
    pub fn new(mac: Mac, name: &str, schedules: Vec<ByteLog>) -> ByteHost {
        ByteHost {
            base: BaseDevice::new(mac, name, 1),
            schedules,
            receives: Vec::new(),
        }
    }
}

impl Device for ByteHost {
    fn base(&self) -> &BaseDevice {
        &self.base
    }

    fn base_mut(&mut self) -> &mut BaseDevice {
        &mut self.base
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) -> Res<()> {
        while let Some((port, x)) = self.base.pop_rbuf() {
            self.receives.push(ByteLog { t: ctx.t, port, x });
        }
        for ByteLog { t, port, x } in &self.schedules {
            if *t == ctx.t {
                self.base.push_sbuf((*port, *x));
            }
        }
        Ok(())
    }
}

pub struct Network {
    devices: Vec<Box<dyn Device>>,
    connections: Vec<Connection>,
}

impl Network {
    pub fn new(devices: Vec<Box<dyn Device>>, medias: Vec<Connection>) -> Network {
        Network {
            devices,
            connections: medias,
        }
    }

    fn connect(&mut self, mac0: Mac, port0: Port, mac1: Mac, port1: Port) -> Res<()> {
        let d = self.get_device(mac0)?;
        let num_ports = d.get_num_ports();
        let name = d.get_name().to_string();
        if port0.value >= num_ports.try_into().unwrap() {
            return Err(Error::NetworkConnectFailed {
                mac0,
                mac1,
                msg: format!(
                    "{}({}):{} port excced num_port({})",
                    name,
                    mac0.value,
                    port0.value,
                    d.get_num_ports(),
                ),
            });
        }

        if self
            .connections
            .iter()
            .any(|c| c.mac0 == mac0 && c.port0 == port0)
        {
            return Err(Error::NetworkConnectFailed {
                mac0,
                mac1,
                msg: format!("{}:{} port already connected", mac0.value, port0.value,),
            });
        }

        if mac0 == mac1 {
            return Err(Error::NetworkConnectFailed {
                mac0,
                mac1,
                msg: "same mac address".to_string(),
            });
        }

        self.connections.push(Connection {
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

    fn find_connection(&self, mac: Mac, port: Port) -> Res<(Mac, Port)> {
        self.connections
            .iter()
            .find(|c| c.mac0 == mac && c.port0 == port)
            .map(|c| (c.mac1, c.port1))
            .ok_or(Error::ConnectionNotFound { mac, port })
    }

    fn update(&mut self, t: usize) -> Res<()> {
        let disp = crate::output::is_byte_level();
        if disp {
            print!("{:>2}: ", t);
        }
        for idx in 0..self.devices.len() {
            let d = &mut self.devices[idx];
            if let Some((src_port, x)) = d.pop_sbuf() {
                let src_mac = d.get_mac();
                let (dst_mac, dst_port) = self.find_connection(src_mac, src_port)?;
                if disp {
                    print!(
                        "{:}:{:} -> {:}:{:} : 0x{:0>2X}     ",
                        src_mac.value, src_port.value, dst_mac.value, dst_port.value, x
                    );
                }
                self.get_device(dst_mac)?.push_rbuf(dst_port, x);
            }
        }
        for d in &mut self.devices {
            let ctx = UpdateContext { t };
            d.update(&ctx)?;
        }
        if disp {
            println!("");
        }
        Ok(())
    }

    pub fn run(&mut self, maxt: usize) -> Res<()> {
        if crate::output::is_byte_level() {
            println!(" t: src -> dst : x");
        } else if crate::output::is_frame_level() {
            println!(" t: device    : action : frame");
        }
        
        for t in 0..maxt {
            self.update(t)?;
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
        ByteLog {
            t: 0,
            port: Port::new(0),
            x: 0x01,
        },
        ByteLog {
            t: 1,
            port: Port::new(0),
            x: 0x02,
        },
        ByteLog {
            t: 2,
            port: Port::new(0),
            x: 0x03,
        },
        ByteLog {
            t: 3,
            port: Port::new(0),
            x: 0x04,
        },
    ];
    let host1 = Box::new(ByteHost::new(mac1, "host1", schedules));
    let mac2 = Mac::new(25);
    let host2 = Box::new(ByteHost::new(mac2, "host2", vec![]));

    let mut nw = Network::new(vec![repeater, host1, host2], vec![]);
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
        assert_eq!(
            log[0],
            ByteLog {
                t: 2,
                port: Port::new(0),
                x: 0x01
            }
        );
        assert_eq!(
            log[2],
            ByteLog {
                t: 4,
                port: Port::new(0),
                x: 0x03
            }
        );
    }
}
