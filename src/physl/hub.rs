use super::{device::{DeviceOperation, Device, DeviceContext}, physl_error::Res};

pub struct Hub {
    store_size: usize,
}

impl Hub {
    pub fn new(mac: usize, name: &str, num_ports: usize, store_size: usize) -> Device {
        Device::new(mac, name, num_ports, Box::new(Hub { store_size }))
    }
}

impl DeviceOperation for Hub {
    fn apply(&mut self, ctx: &DeviceContext, port: usize, rbuf: &Vec<u8>) -> Res<Vec<(usize, Vec<u8>)>> {
        let mut res = Vec::new();
        let rlen = rbuf.len();
        if rlen >= self.store_size {
            for p2 in 0..ctx.num_ports {
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

    fn update(&mut self, _ctx: &DeviceContext) -> Res<Vec<(usize, Vec<u8>)>> {
        Ok(Vec::new())
    }
}

