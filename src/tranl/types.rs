use crate::netwl::IpAddr;

use super::super::types::*;

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Closed,
    Listening,
    SynSent,
    SynAckSent,
    Established,
    DataReceiving,
    DataSent,
    FinSent,
    FinAckSent,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Inst {
    Socket(u32),
    Connect(u32, IpAddr, TPort), // client
    Listen(u32, TPort), // server
    Close(u32),
    Send(u32, String),
    Recv(u32),
}

pub struct Socket {
    pub state: State,
    pub dst_port: TPort,
    pub dst_ip: IpAddr,
}

impl Socket {
    pub fn new() -> Socket {
        Socket {
            state: State::Closed,
            dst_port: TPort::new(0),
            dst_ip: IpAddr::new(0),
        }
    }

    pub fn set_dst(&mut self, dst_port: TPort, dst_ip: IpAddr) {
        self.dst_port = dst_port;
        self.dst_ip = dst_ip;
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
}