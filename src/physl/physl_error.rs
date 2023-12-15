use std::fmt;

#[derive(Debug)]
pub enum PhysicalError {
    InvalidPort {mac: usize, name: String, port: usize},
    DeviceNotFound { mac: usize },
}

impl fmt::Display for PhysicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PhysicalError::InvalidPort { mac, name, port } =>
                write!(f, "Error on device {}({}). Invalid Port: {}", name, mac, port),
            PhysicalError::DeviceNotFound {mac} =>
                write!(f, "Device not found: {}", mac),
        }
    }
}

pub type Res<T> = Result<T, PhysicalError>;
