use std::collections::VecDeque;

use super::physl_error::{PhyslError, Res};

pub struct Device {
	mac: usize,
	name: String,
    num_ports: usize,
	receive_buf: Vec<Vec<u8>>,
	send_buf: Vec<VecDeque<u8>>,
	device_op:  Box<dyn DeviceOperation>,
}

pub struct DeviceContext {
    pub t: usize, 
    pub mac: usize,
    pub name: String,
    pub num_ports: usize,
}

// type PortedBuf = Vec<(usize, Vec<u8>)>;
pub trait DeviceOperation {
    // FIXME: rename
    // event handler for byte arrival for each port
    fn apply(&mut self, ctx: &DeviceContext, rbuf: &mut Vec<Vec<u8>>, sbuf: &mut Vec<VecDeque<u8>>) -> Res<()>;
    // event handler for time tick
    // fn update(&mut self, ctx: &DeviceContext, sbuf: &mut Vec<Vec<u8>>) -> Res<()>;
}

impl Device {	
    pub fn new(mac: usize, name: &str, num_ports: usize, device_fn: Box<dyn DeviceOperation>) -> Device {
        Device {
            mac,
            name: String::from(name),
            num_ports,
            receive_buf: vec![Vec::new(); num_ports],
            send_buf: vec![VecDeque::new(); num_ports],
            device_op: device_fn,
        }
    }

    pub fn get_mac(&self) -> usize {
        self.mac
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn get_context(&self, t: usize) -> DeviceContext {
        DeviceContext {
            t,
            mac: self.mac,
            name: self.name.clone(),
            num_ports: self.num_ports,
        }
    }

    pub fn update(&mut self, t: usize) -> Res<()> {
        let ctx = self.get_context(t);
        // let (, res) = self.device_op.update(&ctx)?;
        // for (port, sbuf) in res {
        //     for b in sbuf {
        //         self.send_buf[port].push_back(b);
        //     }
        // }

        // FIXME: avoid clone
        let mut rbuf = self.receive_buf.clone();
        let mut sbuf = self.send_buf.clone();
        self.device_op.apply(&ctx, &mut rbuf, &mut sbuf)?;
        self.receive_buf = rbuf;
        self.send_buf = sbuf;
        Ok(())
        // let res = {
        //     let ctx = self.get_context(t);
        //     let dfn = &mut self.device_op;
        //     dfn.apply(&ctx, port, &rbuf)
        // };        

        // for port in 0..self.num_ports {
		// 	// let rbuf = &self.receive_bufs[port];
        //     // FIXME: avoid clone
        //     let rbuf = self.receive_buf[port].clone();
		// 	let res = {
        //         let ctx = self.get_context(t);
        //         let dfn = &mut self.device_op;
        //         dfn.apply(&ctx, port, &rbuf)
        //     };
        //     match res {
        //         Ok(res) if res.is_empty() => {},
        //         Ok(res) => {
        //             self.receive_buf[port].clear();
        //             for (port, sbuf) in res {
        //                 for b in sbuf {
        //                     self.send_buf[port].push_back(b);
        //                 }
        //             }
        //         },
        //         Err(PhyslError::InvalidBytes) => {
        //             self.receive_buf[port].clear();
        //         },
        //         Err(e) => return Err(e),

        //     }
		// }
        // Ok(())
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
        self.receive_buf[port].push(value);
        Ok(())
    }

    fn check_port(&self, port: usize) -> Res<()> {
        if port >= self.num_ports {
            return Err(PhyslError::InvalidPort { name: self.name.to_string(), mac: self.mac, port })
        }
        Ok(())
    }

    pub fn get_receive_buf(&self, port: usize) -> Res<&Vec<u8>> {
        self.check_port(port)?;
        Ok(&self.receive_buf[port])
    }
}

#[cfg(test)]
mod tests {
    use crate::physl::{host::Host, hub::Hub};

    use super::*;

    #[test]
    fn test_device() -> Res<()> {
        let mut host_a = Host::new(0, "HostA");
        let mut host_b = Host::new(0, "HostB");
        let mut hub = Hub::new(1, "Hub", 2, 2);
        host_a.push_to_send(0, &[1, 2, 3, 4])?;
        for t in 0..4 {
            let x = host_a.send(0)?;
            let x = x.unwrap();
            println!("t:{}, Sent: {}", t, x);
            hub.receive(0, x)?;
            hub.update(t)?;
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
        let mut host_a = Host::new(0, "HostA");
        let mut host_b = Host::new(0, "HostB");
        let mut hub = Hub::new(1, "Hub", 2, 2);
        host_a.push_to_send(0, &[1, 2, 3, 4])?;
        for t in 0..6 {
            host_a.update(t)?;
            host_b.update(t)?;
            hub.update(t)?;

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