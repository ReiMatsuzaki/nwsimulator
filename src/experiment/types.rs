use std::fmt;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mac(u64);

impl Mac {
    pub fn new(value: u64) -> Mac {
        Mac(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(u32);

impl Port {
    pub fn new(value: u32) -> Port {
        Port(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpAddr {
    pub value: u32,
}

impl IpAddr {
    pub fn new(value: u32) -> IpAddr {
        IpAddr { value }
    }
}

#[derive(Debug)]
pub enum Error {
    // InvalidPort {mac: Mac, name: String, port: Port},
    // DeviceNotFound { mac: Mac },
    DecodeFailed { payload: Vec<u8>, msg: String }
    // LinklError { e: LinklError },
    // InvalidBytes,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Error::InvalidPort { mac, name, port } =>
            //     write!(f, "Error on device {}({}). Invalid Port: {}", name, mac.value(), port.value()),
            // Error::DeviceNotFound {mac} =>
            //     write!(f, "Device not found: {}", mac.value()),
            Error::DecodeFailed { payload, msg } => 
            write!(f, "Decode failed: {}. payload={:?}", msg, payload),
            // Error::LinklError {e} => 
                // write!(f, "LinklError: {:?}", e),
            // PhyslError::InvalidBytes => 
            //     write!(f, "InvalidBytes"),
        }
    }
}

pub type Res<T> = Result<T, Error>;

