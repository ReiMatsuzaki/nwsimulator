use std::{collections::VecDeque, fmt};

pub struct Device {
	mac: usize,
	name: String,
    num_ports: usize,
	receive_buf: Vec<VecDeque<u8>>,
	send_buf: Vec<VecDeque<u8>>,
	device_op:  Box<dyn DeviceOperation>,
}

pub trait DeviceOperation {
    fn apply(&mut self, mac: usize, num_ports: usize, port: usize, rbuf: &VecDeque<u8>) -> Res<Vec<(usize, Vec<u8>)>>;
}

// pub enum DeviceKind { 
//     Host,
//     Hub(Hub)
// }

pub struct Hub {
    store_size: usize,
}
impl DeviceOperation for Hub {
    fn apply(&mut self, _: usize, num_ports: usize, port: usize, rbuf: &VecDeque<u8>) -> Res<Vec<(usize, Vec<u8>)>> {
        let mut res = Vec::new();
        let rlen = rbuf.len();
        if rlen >= self.store_size {
            for p2 in 0..num_ports {
                if p2 != port {
                    let mut sbuf = Vec::new();
                    for i in 0..rlen {
                        let x = rbuf[i];
                        sbuf.push(x);
                    }
                    res.push((p2, sbuf));
                }
            }
        }
        Ok(res)
    }
}

pub struct Host {}
impl DeviceOperation for Host {
    fn apply(&mut self, _mac: usize, _num_ports: usize, _port: usize, _rbuf: &VecDeque<u8>) -> Res<Vec<(usize, Vec<u8>)>> {
        Ok(Vec::new())
    }
}

#[derive(Debug)]
pub struct DeviceError {
    mac: usize,
    name: String,
    kind: DeviceErrorKind,
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self.kind {
            DeviceErrorKind::InvalidPort { port } => 
                format!("Invalid Port: {}", port),
        };
        write!(f, "Error on Device {} ({}): {}", self.mac, self.name, message)
    }
}

#[derive(Debug)]
pub enum DeviceErrorKind {
    InvalidPort {port: usize},
}

pub type Res<T> = Result<T, DeviceError>;

impl Device {	
    pub fn new(mac: usize, name: &str, num_ports: usize, device_fn: Box<dyn DeviceOperation>) -> Device {
        Device {
            mac,
            name: String::from(name),
            num_ports,
            receive_buf: vec![VecDeque::new(); num_ports],
            send_buf: vec![VecDeque::new(); num_ports],
            device_op: device_fn,
        }
    }

    pub fn new_host(mac: usize, name: &str) -> Device {
        Self::new(mac, name, 1, Box::new(Host {  }))
    }

    pub fn new_hub(mac: usize, name: &str, num_ports: usize, store_size: usize) -> Device {
        Self::new(mac, name, num_ports, Box::new(Hub { store_size }))
    }

    fn error(&self, kind: DeviceErrorKind) -> DeviceError {
        DeviceError {
            mac: self.mac,
            name: self.name.clone(),
            kind,
        }
    }

    pub fn get_mac(&self) -> usize {
        self.mac
    }

    pub fn update(&mut self) -> Res<()> {
        for port in 0..self.num_ports {
			// let rbuf = &self.receive_bufs[port];
            // FIXME: avoid clone
            let rbuf = self.receive_buf[port].clone();
			let res = {
                let mac = self.mac;
                let np = self.num_ports;
                let dfn = &mut self.device_op;
                dfn.apply(mac, np, port, &rbuf)?
            };
            // FIXME: avoid clone
            self.receive_buf[port] = rbuf.clone();
			if !res.is_empty() {
				self.receive_buf[port].clear();
                for (port, sbuf) in res {
                    for b in sbuf {
                        self.send_buf[port].push_back(b);
                    }
                }
			}            
		}
        Ok(())
    }

    // FIXME: impl in host
    pub fn push_to_send(&mut self, port: usize, bytes: &[u8]) -> Res<()> {
        self.check_port(port)?;
        for b in bytes {
            self.send_buf[port].push_back(*b);
        }
        Ok(())
    }

    pub fn send(&mut self, port: usize) -> Res<Option<u8>> {
        self.check_port(port)?;
        Ok(self.send_buf[port].pop_front())
    }

    pub fn receive(&mut self, port: usize, value: u8) -> Res<()> {
        self.check_port(port)?;
        self.receive_buf[port].push_back(value);
        Ok(())
    }

    fn check_port(&self, port: usize) -> Res<()> {
        if port >= self.num_ports {
            return Err(self.error(DeviceErrorKind::InvalidPort { port }))
        }
        Ok(())
    }

    pub fn get_receive_buf(&self, port: usize) -> Res<&VecDeque<u8>> {
        self.check_port(port)?;
        Ok(&self.receive_buf[port])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device() -> Res<()> {
        let mut host_a = Device::new_host(0, "HostA");
        let mut host_b = Device::new_host(0, "HostB");
        let mut hub = Device::new_hub(1, "Hub", 2, 2);
        host_a.push_to_send(0, &[1, 2, 3, 4])?;
        for t in 0..4 {
            let x = host_a.send(0)?;
            let x = x.unwrap();
            println!("t:{}, Sent: {}", t, x);
            hub.receive(0, x)?;
            hub.update()?;
        }
        for t in 0..4 {
            let x = hub.send(1)?;
            let x = x.unwrap();
            host_b.receive(0, x)?;
            println!("t:{}, Received: {}", t, x);
            assert_eq!(t+1, x);
        }
        Ok(())
    }

    #[test]
    fn test_device_2() -> Res<()> {
        let mut host_a = Device::new_host(0, "HostA");
        let mut host_b = Device::new_host(0, "HostB");
        let mut hub = Device::new_hub(1, "Hub", 2, 2);
        host_a.push_to_send(0, &[1, 2, 3, 4])?;
        for t in 0..6 {
            host_a.update()?;
            host_b.update()?;
            hub.update()?;

            if let Some(a) = host_a.send(0)? {
                hub.receive(0, a)?;
            }
            if let Some(b) = host_b.send(0)? {
                hub.receive(1, b)?;
            }
            if let Some(c) = hub.send(0)? {
                host_a.receive(0, c)?;
            }
            if let Some(d) = hub.send(1)? {
                println!("t: {}, host b received: {}", t, d);
                host_b.receive(0, d)?;
            }
        }
        assert_eq!(0, host_a.send_buf[0].len());
        assert_eq!(0, hub.receive_buf[0].len());
        assert_eq!(0, hub.receive_buf[1].len());
        assert_eq!(4, host_b.receive_buf[0].len());

        Ok(())
    }    
}