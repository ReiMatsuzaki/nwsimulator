use std::collections::VecDeque;

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
    fn apply(&mut self, ctx: &DeviceContext, rrbuf: &mut Vec<Vec<u8>>, sbuf: &mut Vec<VecDeque<u8>>) -> Res<()> {
        // FIXME: rrbuf should be Vec<VecDeque<u8>>
        for port in 0..ctx.num_ports {
            let rbuf = &mut rrbuf[port];
            let rlen = rbuf.len();
            if rlen >= self.store_size {
                for i in 0..rlen {
                    let x = rbuf[i];
                    for dst_port in 0..ctx.num_ports {
                        if dst_port != port {
                            sbuf[dst_port].push_back(x);
                        }
                    }
                }
                rbuf.clear();
            }
        }
        Ok(())
    }
}

