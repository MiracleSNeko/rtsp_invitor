use bytes::{Buf, BytesMut};
use sscanf::scanf;
use std::{
    collections::HashMap,
    io::{Cursor, Write},
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, Error, ErrorKind};

use crate::{describe_authenticate_request, describe_request, option_request};

/// Defining an enumeration of the possible methods that can be used in an RTSP request.
pub(crate) enum RtspMethod {
    Option,
    Describe,
    Setup,
    Play,
    Teardown,
}

type RtspHeaderMap = HashMap<String, String>;

pub(crate) enum RtspFrame {
    RtspRequest {
        method: RtspMethod,
        url: String,
        c_seq: u16,
        headers: RtspHeaderMap,
    },
    RtspResponse {
        status_code: u16,
        reason_phrase: String,
        c_seq: u16,
        body: String,
        headers: RtspHeaderMap,
    },
}

impl RtspFrame {
    pub(crate) fn assemble_request(&self, buf: &mut [u8]) -> Result<usize, Error> {
        let mut cursor = Cursor::new(buf);

        match self {
            RtspFrame::RtspRequest {
                method,
                url,
                c_seq,
                headers,
            } => {
                let request = match method {
                    RtspMethod::Option => {
                        option_request!(url, c_seq)
                    }
                    RtspMethod::Describe => {
                        // Received DESCRIBE response "RTSP/1.0 401 Unauthorized\r\n"
                        if let Some(authorization) = headers.get(&String::from("Authorization")) {
                            describe_authenticate_request!(url, c_seq, authorization)
                        } else {
                            describe_request!(url, c_seq)
                        }
                    }
                    _ => todo!(),
                };
                cursor.write_all(request.as_bytes())?;
                Ok(request.len())
            }
            RtspFrame::RtspResponse { .. } => Err(Error::new(
                ErrorKind::InvalidData,
                "Cannot assemble response frame",
            )),
        }
    }

    pub(crate) fn check_response(buf: &BytesMut) -> Result<(), Error> {
        // Check if the first line is a valid RTSP response
        if &buf[0..9] == b"RTSP/1.0 " {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Invalid RTSP response"))
        }
    }

    pub(crate) async fn parse_response(cursor: &mut Cursor<&[u8]>) -> Result<(Self, usize), Error> {
        let mut status_code = String::new();
        cursor.read_line(&mut status_code).await?;

        let mut reason_phrase = String::new();
        cursor.read_line(&mut reason_phrase).await?;

        // General header:
        //      C_Seq: <u16>
        let mut generic_header = String::new();
        cursor.read_line(&mut generic_header).await?;
        let c_seq = scanf!(generic_header, "CSeq: {}", u16).unwrap();

        // Require header:
        //      <header>: <content>
        let mut headers = RtspHeaderMap::new();
        loop {
            let mut line = String::new();
            cursor.read_line(&mut line).await?;

            if line.is_empty() {
                break;
            }

            let mut parts = line.splitn(2, ':');
            let header = parts.next().unwrap().trim();
            let content = parts.next().unwrap().trim();

            headers.insert(header.to_string(), content.to_string());
        }

        // SDP body
        let mut body = String::new();
        cursor.read_to_string(&mut body).await?;

        Ok((
            RtspFrame::RtspResponse {
                status_code: status_code.parse::<u16>().unwrap(),
                reason_phrase: reason_phrase.to_string(),
                c_seq,
                headers,
                body,
            },
            cursor.position() as usize,
        ))
    }
}
