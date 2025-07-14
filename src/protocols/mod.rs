use crate::Result;

mod resp;

#[derive(Debug)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Debug)]
pub enum Response {
    Value(String),
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

//pub trait ServerProtocol {
//    fn handle(&self, req: Request) -> Response;
//}
//
//pub trait ClientProtocol {
//    fn send(&mut self, req: Request) -> Result<Response>;
//}

pub struct Protocol;

impl Protocol {
    pub fn build() -> Box<dyn ProtocolTrait> {
        // let _config = Config::from_file("../config/config.toml");
        Box::new(resp::RespProtocol)
    }
}
