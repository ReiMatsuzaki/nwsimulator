use std::fmt;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mac { pub value: u64 }

impl Mac {
    pub fn new(value: u64) -> Mac {
        Mac { value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Port { pub value: u32 }

impl Port {
    pub fn new(value: u32) -> Port {
        Port { value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IpAddr {
    pub value: u32,
}

// impl IpAddr {
//     pub fn new(value: u32) -> IpAddr {
//         IpAddr { value }
//     }
// }

#[derive(Debug)]
pub enum Error {
    NotEnoughBytes,
    // InvalidPort {mac: Mac, name: String, port: Port, msg: String},
    DeviceNotFound { mac: Mac },
    // DecodeFailed { payload: Vec<u8>, msg: String },
    NetworkConnectFailed { mac0: Mac, mac1: Mac, msg: String },
    ConnectionNotFound { mac: Mac, port: Port },
    // LinklError { e: LinklError },
    InvalidBytes { msg: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotEnoughBytes => 
                write!(f, "Not enough byte"),
            // Error::InvalidPort { mac, name, port, msg } =>
            //     write!(f, "Invalid port on device {}({}). port={}, msg={}", name, mac.value, port.value, msg),
            Error::DeviceNotFound {mac} =>
                write!(f, "Device not found: {}", mac.value),
            // Error::DecodeFailed { payload, msg } => 
            //     write!(f, "Decode failed: {}. payload={:?}", msg, payload),
            Error::NetworkConnectFailed { mac0, mac1, msg } =>
                write!(f, "Network connect faild: {} - {}. {}", mac0.value, mac1.value, msg),
            Error::ConnectionNotFound { mac, port } =>
                write!(f, "Connection not found: mac={}, port={}", mac.value, port.value),
                // Error::LinklError {e} => 
                // write!(f, "LinklError: {:?}", e),
            Error::InvalidBytes { msg } => 
                write!(f, "Invalid bytes, {}", msg),
        }
    }
}

pub type Res<T> = Result<T, Error>;

