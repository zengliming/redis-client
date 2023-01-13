use bytes::BytesMut;
use crate::protocol::ReplayType::{Bulk, Error, Int, MultiBulk, Status};

pub fn gen_req(args: &[&str]) -> String {
    let mut cmd = String::from("*");
    cmd.push_str(&args.len().to_string());
    cmd.push_str("\r\n");
    for arg in args.iter() {
        let arg_len = arg.len();
        cmd.push_str("$");
        cmd.push_str(&arg_len.to_string());
        cmd.push_str("\r\n");
        cmd.push_str(arg);
        cmd.push_str("\r\n");
    }
    cmd
}

#[derive(Debug)]
pub struct Replay {
    pub replay_type: ReplayType,
    pub replay_data: Vec<String>,
}

#[derive(Debug)]
pub enum ReplayType {
    Status,
    Error,
    Int,
    Bulk,
    MultiBulk,
}

pub fn parse_resp(resp: BytesMut) -> Replay {
    let result = String::from_utf8(resp.freeze().to_vec()).unwrap();
    let splits: Vec<&str> = result.split("\r\n").collect();
    let len = splits.len();
    let mut resp_type = Status;
    let mut parse_result: Vec<String> = Vec::new();
    if len > 0 {
        let head = splits[0];
        let flag = &head[0..1];
        let state = &head[1..head.len()];
        if flag == "+" {
            resp_type = Status;
            parse_result.push(state.to_string());
        } else if flag == "-" {
            resp_type = Error;
            parse_result.push(state.to_string());
        } else if flag == ":" {
            resp_type = Int;
            parse_result.push(splits[1].to_string());
        } else if flag == "$" {
            resp_type = Bulk;
            if flag == "-1" {
                parse_result.push("".to_string());
            } else {
                parse_result.push(splits[1].to_string());
            }
        } else if flag == "*" {
            resp_type = MultiBulk;
            let mut i = 2;
            loop {
                if i >= len {
                    break;
                }
                parse_result.push(splits[i].to_string());
                i = i + 2;
            }
        }
    }

    Replay {
        replay_type: resp_type,
        replay_data: parse_result,
    }
}