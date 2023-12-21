use std::any::Any;
use std::collections::VecDeque;

use crate::netwl::IP;

use super::super::physl::{Device, BaseDevice};
use super::super::netwl::{BaseIpDevice, IpAddr, SubnetMask};
use super::super::types::*;
use super::{TCP, TcpContent};
use super::types::*;

pub struct TcpHost {
    ip_base: BaseIpDevice,
    insts: VecDeque<Inst>,

    // socket
    socket: Option<Socket>
}

impl TcpHost { 
    pub fn new(mac: Mac, name: &str, ip_addr: IpAddr, subnet_mask: SubnetMask) -> TcpHost {
        TcpHost {
            ip_base: BaseIpDevice::new(mac, name, vec![ip_addr], subnet_mask),
            socket: None,
            insts: VecDeque::new(),
        }
    }

    pub fn build(mac: Mac, name: &str, ip_addr: IpAddr, subnet_mask: SubnetMask) -> Box<TcpHost> {
        let host = TcpHost::new(mac, name, ip_addr, subnet_mask);
        Box::new(host)
    }

    fn get_ip_addr(&self) -> IpAddr {
        self.ip_base.get_ip_addr(Port::new(0)).unwrap()
    }

    // FIXME: rename
    fn handle_insts(&mut self, _ctx: &UpdateContext) -> Option<(IpAddr, TCP)> {
        let inst = self.insts.front()?.clone();
        match &mut self.socket {
            None => match inst {
                Inst::Connect(_sid, ip_addr, port) => {
                    let s = Socket::new(State::SynSent, port, ip_addr);
                    self.socket = Some(s);
                    let tcp = TCP::new_syn(TPort::new(0),  port, 0);
                    self.insts.pop_front();
                    Some((ip_addr.clone(), tcp))
                },
                Inst::Listen(_sid, _port) => {
                    let s = Socket::new(State::Listening, TPort::new(0), IpAddr::new(0));
                    self.socket = Some(s);
                    self.insts.pop_front();
                    None
                }
                _ => None
            }
            Some(s) if s.state == State::Established => {
                match inst {
                    Inst::Send(_sid, msg) => {
                        s.set_state(State::DataSent);
                        let payload = msg.as_bytes().to_vec();
                        let tcp = TCP::new_data(TPort::new(0), s.dst_port, 0, 0, 0, payload);
                        self.insts.pop_front();
                        Some((s.dst_ip, tcp))
                    }
                    Inst::Recv(_sid)  => {
                        s.set_state(State::DataReceiving);
                        self.insts.pop_front();
                        None            
                    }
                    Inst::Close(_sid) => {
                        s.set_state(State::FinSent);
                        let tcp = TCP::new_fin(TPort::new(0), s.dst_port, 0, 0);
                        self.insts.pop_front();
                        Some((s.dst_ip, tcp))
                    }
                    _ => None
                }
            }
            _ => None
        }
   }

    pub fn handle_tcp(&mut self, tcp: &TCP, _ctx: &UpdateContext) -> Res<Option<TCP>> {
        let cnt = &tcp.content;
        match &mut self.socket {
            None => Err(Error::InvalidTcpReceived { msg: "state is NoConnection".to_string() }),
            Some(s) => match s.state {
                State::Listening => {
                    if let TcpContent::Syn = cnt {
                        s.state = State::SynAckSent;
                        let tcp = TCP::new_synack(tcp.dst, tcp.src, 0);
                        Ok(Some(tcp))
                    } else {
                        Err(Error::InvalidTcpReceived { msg: format!("state is Listening but receive {:?}", cnt) })
                    }
                }
                State::SynSent => {
                    if TcpContent::SynAck == *cnt {
                        s.state = State::Established;
                        let tcp = TCP::new_ack(tcp.dst, tcp.src, 0, 0);
                        Ok(Some(tcp))
                    } else {
                        Err(Error::InvalidTcpReceived { msg: format!("state is SynSent but receive {:?}", cnt) })
                    }
                },
                State::SynAckSent => {
                    if TcpContent::Ack == *cnt {
                        s.state = State::Established;
                        Ok(None)
                    } else {
                        Err(Error::InvalidTcpReceived { msg: format!("state is SynAckSent but receive {:?}", cnt) })
                    }
                }
                State::Established => {
                    if TcpContent::Fin == *cnt {
                        s.state = State::FinAckSent;
                        let tcp = TCP::new_finack(tcp.dst, tcp.src, 0, 0);
                        Ok(Some(tcp))
                    } else {
                        Err(Error::InvalidTcpReceived { msg: format!("state is established but receive {:?}", cnt) })
                    }
                },
                State::DataReceiving => {
                    if TcpContent::Data == *cnt {
                        s.state = State::Established;
                        let tcp = TCP::new_ack(tcp.dst, tcp.src, 0, 0);
                        Ok(Some(tcp))
                    } else {
                        Err(Error::InvalidTcpReceived { msg: format!("state is DataReceiving but receive {:?}", cnt) })
                    }
                }
                State::DataSent => {
                    if TcpContent::Ack == *cnt {
                        s.state = State::Established;
                        Ok(None)
                    } else {
                        Err(Error::InvalidTcpReceived { msg: format!("state is DataSent but receive {:?}", cnt) })
                    }
                },
                State::FinSent => {
                    if TcpContent::FinAck == *cnt {
                        // FIXME: close socket
                        let tcp = TCP::new_ack(tcp.dst, tcp.src, 0, 0);
                        Ok(Some(tcp))
                    } else {
                        Err(Error::InvalidTcpReceived { msg: format!("state is FinSent but receive {:?}", cnt) })
                    }
                }
                State::FinAckSent => {
                    if TcpContent::Ack == *cnt {
                        // FIXME: close socket
                        Ok(None)
                    } else {
                        Err(Error::InvalidTcpReceived { msg: format!("state is FinAckSent but receive {:?}", cnt) })
                    }
                }
            }
        }
    }

    pub fn add_inst(&mut self, inst: Inst) {
        self.insts.push_back(inst);
    }

    fn recv_ip(&mut self, ctx: &UpdateContext) -> Res<Option<IP>> {
        let x = self.ip_base.pop_rbuf(ctx)?
        .and_then(|p|
            match p {
                crate::netwl::NetworkProtocol::IP(ip) => Some(ip),
                _ => None
            }
        );
        Ok(x)
    }

    fn send_ip(&mut self, ip: IP, ctx: &UpdateContext) -> Res<()> {
        self.ip_base.push_sbuf(crate::netwl::NetworkProtocol::IP(ip), ctx)?;
        Ok(())
    }
}

impl Device for TcpHost {
    fn base(&self) -> &BaseDevice {
        self.ip_base.base()
    }

    fn base_mut(&mut self) -> &mut BaseDevice {
        self.ip_base.base_mut()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn update(&mut self, ctx: &UpdateContext) -> Res<()> {
        if let Some((ip_addr, tcp)) = self.handle_insts(&ctx) {
            let payload = tcp.encode();
            let ip = IP::new_byte(self.get_ip_addr(), ip_addr, payload);
            self.send_ip(ip, ctx)?;
        }

        while let Some(ip) = self.recv_ip(ctx)? {
            let tcp = TCP::decode(&ip.payload_as_bytes())?;
            if let Some(tcp) = self.handle_tcp(&tcp, ctx)? {
                let payload = tcp.encode();
                let ip = IP::new_byte(ip.dst, ip.src, payload);
                self.send_ip(ip, ctx)?;
            }
        }
        Ok(())
    }
}
