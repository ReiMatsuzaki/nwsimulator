use super::super::types::*;
use super::device::*;

pub struct Connection {
    pub mac0: Mac,
    pub port0: Port,
    pub mac1: Mac,
    pub port1: Port,
}

pub struct Network {
    devices: Vec<Box<dyn Device>>,
    connections: Vec<Connection>,
}

impl Network {
    // FIXME: avoid box, remove medias
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
            if let Some((src_port, x)) = d.pop_send() {
                let src_mac = d.get_mac();
                let (dst_mac, dst_port) = self.find_connection(src_mac, src_port)?;
                if disp {
                    print!(
                        "{:}:{:} -> {:}:{:} : 0x{:0>2X}     ",
                        src_mac.value, src_port.value, dst_mac.value, dst_port.value, x
                    );
                }
                self.get_device(dst_mac)?.push_recv(dst_port, x);
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

