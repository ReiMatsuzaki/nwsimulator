
use std::fmt;

use crate::linkl::LinklError;

#[derive(Debug)]
pub enum PhyslError {
    InvalidPort {mac: usize, name: String, port: usize},
    DeviceNotFound { mac: usize },
    LinklError { e: LinklError },
}

impl fmt::Display for PhyslError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhyslError::InvalidPort { mac, name, port } =>
                write!(f, "Error on device {}({}). Invalid Port: {}", name, mac, port),
            PhyslError::DeviceNotFound {mac} =>
                write!(f, "Device not found: {}", mac),
            PhyslError::LinklError {e} => 
                write!(f, "LinklError: {:?}", e),
        }
    }
}

pub type Res<T> = Result<T, PhyslError>;
