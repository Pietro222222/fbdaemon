use bincode::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Messages {
    ///rgb value, index
    Write((u8, u8, u8), usize),
    ///vector of messages
    MVec(Vec<Messages>),
    ///clear screen
    Clear,
}

impl Messages {
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        bincode::serialize(self)
    }
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        bincode::deserialize(&bytes[..])
    }
}
