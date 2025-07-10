use super::*;
use super::{ClientProtocol, Protocol, Request, Response, ServerProtocol};
use crate::common::Result;

pub struct RespProtocol;

impl ProtocolTrait for RespProtocol {
    fn encode_request(&self, req: &Request) -> Vec<u8>;
    fn decode_request(&self, data: &[u8]) -> Result<Request>;

    fn encode_response(&self, res: &Response) -> Vec<u8>;
    fn decode_response(&self, data: &[u8]) -> Result<Response>;
}

impl ServerProtocol for RespProtocol {
    fn handle(&self, req: Request) -> Response;
}

impl ClientProtocol for RespProtocol {
    fn send(&mut self, req: Request) -> Result<Response>;
}
