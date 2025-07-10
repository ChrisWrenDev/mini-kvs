use crate::Config;

mod binary;

pub trait SerializationTrait {}

pub enum Serialization {
    Binary(binary::Binary),
}

impl Serialization {
    pub fn build(config: &Config) -> Self {
        return Serialization::Binary(binary::Binary);
    }
}
