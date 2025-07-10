use crate::{Config, Result};

mod resp;

pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}

pub enum Response {
    Value(Option<String>),
    Ok,
    NotFound,
    Error(String),
}

pub trait ProtocolTrait {
    fn encode_request(&self, req: &Request) -> Vec<u8>;
    fn decode_request(&self, data: &[u8]) -> Result<Request>;

    fn encode_response(&self, res: &Response) -> Vec<u8>;
    fn decode_response(&self, data: &[u8]) -> Result<Response>;
}

pub trait ServerProtocol {
    fn handle(&self, req: Request) -> Response;
}

pub trait ClientProtocol {
    fn send(&mut self, req: Request) -> Result<Response>;
}

pub enum Protocol {
    RESP(resp::RespProtocol),
}

impl Protocol {
    pub fn build(config: &Config) -> Self {
        return Protocol::RESP(resp::RespProtocol);
    }
}
