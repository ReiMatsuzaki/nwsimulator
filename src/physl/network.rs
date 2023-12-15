use crate::physl::{hub::Hub, host::Host, physl_error::PhyslError};

use super::{device::Device, physl_error::Res};

pub struct Network {
    devices: Vec<Box<Device>>,
    medias: Vec<Media>
}

pub struct Media {
    dnum0: usize,
    port0: usize,
    dnum1: usize,
    port1: usize,
}

impl Network {
    pub fn new() -> Network {
        Network {
            devices: Vec::new(),
            medias: Vec::new(),
        }
    }

    pub fn add_device(&mut self, device: Device) {
        self.devices.push(Box::new(device));
    }

    fn find_device(&self, mac: usize) -> Res<(usize, &Device)> {
        for dnum in 0..self.devices.len() {
            let device: &Box<Device> = &self.devices[dnum];
            if device.get_mac() == mac {
                return Ok((dnum, device));
            }
        }
        Err(PhyslError::DeviceNotFound { mac })
    }

    // FIXME: mac address should be u64
    pub fn connect(&mut self, mac0: usize, port0: usize, mac1: usize, port1: usize) -> Res<()> {
        let (dnum0, _) = self.find_device(mac0)?;
        let (dnum1, _) = self.find_device(mac1)?;
        let m = Media { dnum0, port0, dnum1, port1 };
        self.medias.push(m);
        Ok(())
    }

    pub fn start(&mut self, max_t: usize) -> Res<()> {
        let disp = crate::output::is_byte_level();
        if disp {
            println!("t");
        }
        for t in 0..max_t {
            if disp {
                print!("{:>2}: ", t);
            }
            for device in &mut self.devices {
                device.update(t)?;
            }
            for i in 0..self.medias.len() {
                let media = &self.medias[i];
                self.swap_data(media.dnum0, media.port0, media.dnum1, media.port1)?;
            }
            if disp {
                println!("");
            }
        }
        Ok(())
    }

    // FIXME: rename?
    fn swap_data(&mut self, dnum0: usize, p0: usize, dnum1: usize, p1: usize) -> Result<(), PhyslError> {
        let disp = crate::output::is_byte_level();
        let val0 = self.devices[dnum0].send(p0)?;
        let val1 = self.devices[dnum1].send(p1)?;
        if let Some(value) = val0 {
            self.devices[dnum1].receive(p1, value)?;
            if disp {
                print!("{}({}) -> {}({}) : 0x{:0>2X}   ", 
                self.devices[dnum0].get_name(), p0, 
                self.devices[dnum1].get_name(), p1, value);
            }
        }
        if let Some(value) = val1 {
            self.devices[dnum0].receive(p0, value)?;
            if disp {
                print!("{}({}) -> {}({}) : 0x{:0>2X}   ", 
                self.devices[dnum1].get_name(), p1,
                self.devices[dnum0].get_name(), p0, 
                value);
            }
        }
        Ok(())
    }    

    // FIXME: rename receive_buf to receive_bufs
    pub fn get_receive_buf(&self, mac: usize, port: usize) -> Res<&Vec<u8>> {
        let (_, d) = self.find_device(mac)?;
        d.get_receive_buf(port)
    }
}

pub fn run_main() -> Res<()> {
    crate::output::set_level(crate::output::Level::Byte);
    run_1hub_2host()?;
    Ok(())
}

fn run_1hub_2host() -> Res<Vec<u8>> {
    let mut nw = Network::new();
    let mac_host_a = 1011;
    let mac_host_b = 1012;
    let mac_hub = 1013;
    let mut host_a = Host::new(mac_host_a, "HostA");
    host_a.push_to_send(0, b"Hello").unwrap();
    let host_b = Host::new(mac_host_b, "HostB");
    let hub = Hub::new(mac_hub, "Hub", 2, 2);
    nw.add_device(host_a);
    nw.add_device(host_b);
    nw.add_device(hub);
    nw.connect(mac_host_a, 0, mac_hub, 0)?;
    nw.connect(mac_host_b, 0, mac_hub, 1)?;

    nw.start(10)?;

    let rbuf = nw.get_receive_buf(mac_host_b, 0)?;
    println!("host b received:");
    println!("{:?}", rbuf.iter().map(|x| *x as char).collect::<String>());
    Ok(rbuf.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_nw() {
        let rbuf = run_1hub_2host().unwrap();
        assert_eq!(rbuf, b"Hell");
    }
}