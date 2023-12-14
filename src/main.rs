use std::collections::VecDeque;

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
    Hub {store_size: usize} ,
}

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
            kind: DeviceKind::Hub { store_size },
        }
    }

    fn update(&mut self) {
        match self.kind {
            DeviceKind::Host => {
                let x = self.receive_buf[0].pop_front().unwrap();
                println!("Received: {}", x);
            }
            DeviceKind::Hub {store_size} => {
                for port in 0..self.num_ports {
                    let rlen = self.receive_buf[port].len();
                    if rlen >= store_size {
                        for p2 in 0..self.num_ports {
                            if p2 != port {
                                for _ in 0..rlen {
                                    self.send_buf[p2].push_back(self.receive_buf[port].pop_front().unwrap());
                                }
                            }
                        // for _ in 0..rlen {
                        //     self.send_buf[port].push_back(self.receive_buf[port].pop_front().unwrap());
                        // }
                        }
                    }
                }
            }
        }
    }

    fn push_bytes(&mut self, port: usize, bytes: &[u8]) {
        assert!(port < self.num_ports);
        for b in bytes {
            self.send_buf[port].push_back(*b);
        }
    }

    fn send(&mut self, port: usize) -> u8 {
        assert!(port < self.num_ports);
        self.send_buf[port].pop_front().unwrap()
    }

    fn receive(&mut self, port: usize, value: u8) {
        assert!(port < self.num_ports);
        self.receive_buf[port].push_back(value);
    }
}

fn main() {
    let mut host_a = Device::new_host(0, "HostA");
    let mut host_b = Device::new_host(0, "HostB");
    println!("{}, {}", host_a.mac, host_a.name);
    let mut hub = Device::new_hub(1, "Hub", 2, 2);
    host_a.push_bytes(0, &[1, 2, 3, 4]);
    for t in 0..4 {
        let x = host_a.send(0);
        println!("t:{}, Sent: {}", t, x);
        hub.receive(0, x);
        hub.update();
    }
    for t in 0..4 {
        let x = hub.send(1);
        host_b.receive(0, x);
        println!("t:{}, Received: {}", t, x)
    }
}
