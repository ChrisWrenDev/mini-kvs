use super::*;
use crate::{KvsError, Result};

pub struct RespProtocol;

impl ProtocolTrait for RespProtocol {
    fn encode_request(&self, req: &Request) -> Vec<u8> {
        match req {
            Request::Set { key, value } => serialize("SET", &[key, value]),
            Request::Get { key } => serialize("GET", &[key]),
            Request::Remove { key } => serialize("REMOVE", &[key]),
        }
        .into_bytes()
    }

    fn decode_request(&self, data: &[u8]) -> Result<Request> {
        let parts = deserialize(data)?;
        match parts.as_slice() {
            ["GET", key] => Ok(Request::Get {
                key: key.to_string(),
            }),
            ["SET", key, value] => Ok(Request::Set {
                key: key.to_string(),
                value: value.to_string(),
            }),
            ["REMOVE", key] => Ok(Request::Remove {
                key: key.to_string(),
            }),
            _ => Err("Invalid request format".into()),
        }
    }

    fn encode_response(&self, res: &Response) -> Vec<u8> {
        match res {
            Response::Value(val) => serialize("VALUE", &[val]),
            Response::Ok => serialize("OK", &[]),
            Response::NotFound => serialize("NOT_FOUND", &[]),
            Response::Error(err) => serialize("ERROR", &[err]),
        }
        .into_bytes()
    }

    fn decode_response(&self, data: &[u8]) -> Result<Response> {
        let parts = deserialize(data)?;
        match parts.as_slice() {
            ["VALUE", val] => Ok(Response::Value(val.to_string())),
            ["OK"] => Ok(Response::Ok),
            ["NOT_FOUND"] => Ok(Response::NotFound),
            ["ERROR", err] => Ok(Response::Error(err.to_string())),
            _ => Err(KvsError::Protocol("Invalid response format".into())),
        }
    }
}

impl ServerProtocol for RespProtocol {
    fn handle(&self, req: Request) -> Response;
}

impl ClientProtocol for RespProtocol {
    fn send(&mut self, req: Request) -> Result<Response>;
}

pub fn serialize(command: &str, args: &[&str]) -> String {
    let total_parts = 1 + args.len();
    let mut resp = format!("*{}\r\n", total_parts);
    resp += &format!("${}\r\n{}\r\n", command.len(), command);

    for arg in args {
        resp += &format!("${}\r\n{}\r\n", arg.len(), arg);
    }

    resp
}

pub fn deserialize(data: &[u8]) -> Result<Vec<&str>> {
    let input = std::str::from_utf8(data)?;
    let mut lines = input.split("\r\n").filter(|line| !line.is_empty());

    let header = lines.next().ok_or("Missing array header")?;
    let count: usize = header
        .strip_prefix('*')
        .ok_or("Expected '*' array prefix")?
        .parse()
        .map_err(|_| "Invalid array count")?;

    let mut result = Vec::with_capacity(count);

    for _ in 0..count {
        let len_line = lines.next().ok_or("Missing bulk string header")?;
        let len: usize = len_line
            .strip_prefix('$')
            .ok_or("Expected '$' bulk string prefix")?
            .parse()
            .map_err(|_| "Invalid bulk string length")?;

        let value = lines.next().ok_or("Missing bulk string value")?;
        if value.len() != len {
            return Err("Bulk string length mismatch".into());
        }

        result.push(value);
    }

    Ok(result)
}
