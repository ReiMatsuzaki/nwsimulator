use std::{collections::VecDeque, fmt};

struct Device {
	mac: usize,
	name: String,
    num_ports: usize,
	receive_buf: Vec<VecDeque<u8>>,
	send_buf: Vec<VecDeque<u8>>,
	kind: DeviceKind,
}

enum DeviceKind { 
    Host,
    Hub(Hub)
}

struct Hub {
    store_size: usize,
}

#[derive(Debug)]
struct DeviceError {
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
enum DeviceErrorKind {
    InvalidPort {port: usize},
}

type Res<T> = Result<T, DeviceError>;

impl Device {	
    pub fn new_host(mac: usize, name: &str) -> Device {
        Device {
            mac,
            name: String::from(name),
            num_ports: 1,
            receive_buf: vec![VecDeque::new()],
            send_buf: vec![VecDeque::new()],
            kind: DeviceKind::Host,
        }
    }

    pub fn new_hub(mac: usize, name: &str, num_ports: usize, store_size: usize) -> Device {
        Device {
            mac,
            name: String::from(name),
            num_ports,
            receive_buf: vec![VecDeque::new(); num_ports],
            send_buf: vec![VecDeque::new(); num_ports],
            kind: DeviceKind::Hub(Hub { store_size } ),
        }
    }

    fn error(&self, kind: DeviceErrorKind) -> DeviceError {
        DeviceError {
            mac: self.mac,
            name: self.name.clone(),
            kind,
        }
    }

    fn update(&mut self) -> Res<()> {
        match &self.kind {
            DeviceKind::Host => self.update_host(),
            DeviceKind::Hub(_)  => self.update_hub()
        }
    }

    fn update_host(&mut self) -> Res<()> {
        if let Some(x) = self.receive_buf[0].pop_front() {
            println!("Received: {}", x);
        }
        Ok(())
    }

    fn update_hub(&mut self) -> Res<()> {
        let h = match &self.kind {
            DeviceKind::Hub(h) => h,
            _ => panic!("Not a hub")
        };
        for port in 0..self.num_ports {
            let rlen = self.receive_buf[port].len();
            if rlen >= h.store_size {
                for p2 in 0..self.num_ports {
                    if p2 != port {
                        for _ in 0..rlen {
                            let x = self.receive_buf[port].pop_front().unwrap();
                            self.send_buf[p2].push_back(x);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn push_to_send_buf(&mut self, port: usize, bytes: &[u8]) -> Res<()> {
        self.check_port(port)?;
        for b in bytes {
            self.send_buf[port].push_back(*b);
        }
        Ok(())
    }

    fn send(&mut self, port: usize) -> Res<Option<u8>> {
        self.check_port(port)?;
        Ok(self.send_buf[port].pop_front())
    }

    fn receive(&mut self, port: usize, value: u8) -> Res<()> {
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
}

fn main() {
    fn f() -> Res<()> {
        let mut host_a = Device::new_host(0, "HostA");
        let mut host_b = Device::new_host(0, "HostB");
        println!("{}, {}", host_a.mac, host_a.name);
        let mut hub = Device::new_hub(1, "Hub", 2, 2);
        host_a.push_to_send_buf(0, &[1, 2, 3, 4])?;
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
            println!("t:{}, Received: {}", t, x)
        }
        Ok(())
    }
    f().unwrap();
}
