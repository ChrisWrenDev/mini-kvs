use crate::{Config, Result, SerializationConfig};

mod binary;

pub trait SerializationTrait {}

pub struct Serialization;

impl Serialization {
    pub fn build(config: &Config) -> Result<Box<dyn SerializationTrait>> {
        match config.serialization {
            SerializationConfig::Binary => Ok(Box::new(binary::Binary)),
        }
    }
}
