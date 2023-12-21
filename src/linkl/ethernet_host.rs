use std::collections::VecDeque;

use crate::{types::{Res, Mac, UpdateContext}, physl::{Device, BaseDevice}};

use super::{BaseEthernetDevice, EthernetDevice, EthernetFrame, EthernetLog};
pub struct EthernetHost {
    pub base: BaseEthernetDevice,
    schedules: VecDeque<EthernetLog>,
    handler: Box<dyn Fn(&Vec<u8>) -> Res<Option<Vec<u8>>>>,
}

impl EthernetHost {
    fn new(base: BaseEthernetDevice, handler: Box<dyn Fn(&Vec<u8>) -> Res<Option<Vec<u8>>>>) -> EthernetHost {
        EthernetHost {
            base,
            schedules: VecDeque::new(),
            handler,
        }
    }

    pub fn build_consumer(mac: Mac, name: &str) -> Box<EthernetHost> {
        let base = BaseEthernetDevice::new(mac, name, 1);
        let handler = Box::new(
            |_bytes: &Vec<u8>| {
                Ok(None)
            }
        );
        Box::new(Self::new(base, handler))
    }

    pub fn build_echo(mac: Mac, name: &str) -> Box<EthernetHost> {
        let base = BaseEthernetDevice::new(mac, name, 1);
        let handler = Box::new(
            |bytes: &Vec<u8>| Ok(Some(bytes.clone()))
        );
        Box::new(Self::new(base, handler))
    }

    pub fn add_schedule(&mut self, t: usize, frame: EthernetFrame) {
        self.schedules.push_back(EthernetLog { t, frame });
    }
}

impl EthernetDevice for EthernetHost {
    fn ether_base(&self) -> &BaseEthernetDevice {
        &self.base
    }

    fn ether_base_mut(&mut self) -> &mut BaseEthernetDevice {
        &mut self.base
    }
}

impl Device for EthernetHost {
    fn base(&self) -> &BaseDevice {
        &self.base.base
    }

    fn base_mut(&mut self) -> &mut BaseDevice {
        &mut self.base.base
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) -> Res<()> {
        if let Some(schedule) = self.schedules.front() {
            if schedule.t == ctx.t {
                self.base.push_sbuf(schedule.frame.clone(), ctx);
                self.schedules.pop_front();
            }
        }

        if let Some(frame) = self.pop_rbuf(ctx) {
            if frame.dst == self.get_mac() {
                if let Some(bytes) = (self.handler)(&frame.payload)? {
                    let frame = EthernetFrame::new(frame.src, frame.dst, frame.ethertype, bytes);
                    self.push_sbuf(frame, ctx);
                }
            }
        }
        Ok(())
    }
}

