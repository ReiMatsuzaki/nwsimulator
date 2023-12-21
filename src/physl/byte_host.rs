use super::super::types::*;
use super::device::*;
use std::any::Any;

pub struct ByteHost {
    base: BaseDevice,
    schedules: Vec<ByteLog>,
    rlogs: Vec<ByteLog>,
}

impl ByteHost {
    pub fn new(mac: Mac, name: &str, schedules: Vec<ByteLog>) -> ByteHost {
        ByteHost {
            base: BaseDevice::new(mac, name, 1),
            schedules,
            rlogs: Vec::new(),
        }
    }

    pub fn get_rlogs(&self) -> &Vec<ByteLog> {
        &self.rlogs
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
            self.rlogs.push(ByteLog { t: ctx.t, port, x });
        }
        for ByteLog { t, port, x } in &self.schedules {
            if *t == ctx.t {
                self.base.push_sbuf((*port, *x));
            }
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

impl ByteLog {
    pub fn new(t: usize, port: Port, x: u8) -> ByteLog {
        ByteLog { t, port, x }
    }
}
