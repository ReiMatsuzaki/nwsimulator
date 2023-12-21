use std::collections::VecDeque;

use crate::types::{Mac, Res};
use crate::physl::{BaseDevice, Device, UpdateContext};

use super::{BaseEthernetDevice, EthernetLog, EthernetFrame};


type BytesFn = dyn Fn(&Vec<u8>) -> Res<Option<Vec<u8>>>;

enum DeviceType {
    Host(Box<BytesFn>),
    Hub,
} 
pub struct EthernetSwitch {
    base: BaseEthernetDevice,
    // handler: Box<BytesFn>,
    device_type: DeviceType,
    schedules: VecDeque<EthernetLog>,
}

impl EthernetSwitch {
    fn new(base: BaseEthernetDevice, device_type: DeviceType) -> EthernetSwitch {
        EthernetSwitch {
            base,
            device_type,
            schedules: VecDeque::new(),
        }
    }

    pub fn build_host(mac: Mac, name: &str) -> Box<EthernetSwitch> {
        let base = BaseEthernetDevice::new(mac, name, 1);
        let handler = Box::new(
            |_bytes: &Vec<u8>| Ok(None)
        );
        Box::new(Self::new(base, DeviceType::Host(handler)))
    }

    pub fn build_echo_host(mac: Mac, name: &str) -> Box<EthernetSwitch> {
        let base = BaseEthernetDevice::new(mac, name, 1);
        let handler = Box::new(
            |bytes: &Vec<u8>| Ok(Some(bytes.clone()))
        );
        Box::new(Self::new(base, DeviceType::Host(handler)))
    }

    pub fn build_bridge(mac: Mac, name: &str) -> Box<EthernetSwitch> {
        let base = BaseEthernetDevice::new(mac, name, 2);
        Box::new(Self::new(base, DeviceType::Hub))
    }

    pub fn build_switch(mac: Mac, name: &str, num_ports: usize) -> Box<EthernetSwitch> {
        let base = BaseEthernetDevice::new(mac, name, num_ports);
        Box::new(Self::new(base, DeviceType::Hub))
    }

    pub fn add_schedule(&mut self, t: usize, frame: EthernetFrame) {
        self.schedules.push_back(EthernetLog { t, frame });
    }

    pub fn get_rlog(&self) -> &Vec<EthernetLog> {
        &self.base.rlog
    }

    pub fn get_slog(&self) -> &Vec<EthernetLog> {
        &self.base.rlog
    }
}

impl Device for EthernetSwitch {
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

        // rbuf -> sbuf
        while let Some(frame) = self.base.pop_rbuf(ctx) {
            let dst = frame.dst;
            let src = frame.src;
            match &self.device_type {
                DeviceType::Host(f) => {
                    if dst == self.base.base.get_mac() {
                        if let Some(payload) = (f)(&frame.payload)? {
                            let f = EthernetFrame::new(src, dst, payload.len() as u16, payload);
                            self.base.push_sbuf(f, ctx);
                        }
                    } else {
                        // this frame is not for me. just consume it.
                    }
                },
                DeviceType::Hub => {
                    // this frame is not for me. just forward it.
                    self.base.push_sbuf(frame, ctx);
                }
            }
        }
        Ok(())
    }
}

