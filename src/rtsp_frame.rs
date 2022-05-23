use bytes::BytesMut;
use sscanf::scanf;
use std::{
    collections::HashMap,
    io::{Cursor, Write},
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, Error, ErrorKind, Result};

use crate::{
    describe_authenticate_request, describe_request, option_request, play_request, setup_request,
    teardown_request,
};

/// Defining an enumeration of the possible methods that can be used in an RTSP request.
pub(crate) enum RtspMethod {
    Option,
    Describe,
    Setup,
    Play,
    Teardown,
}

pub(crate) type RtspHeaderMap = HashMap<String, String>;

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
        headers: RtspHeaderMap,
        body: String,
    },
}

impl RtspFrame {
    pub(crate) fn assemble_request(&self, buf: &mut [u8]) -> Result<usize> {
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
                    RtspMethod::Setup => {
                        let port = headers
                            .get(&String::from("Port"))
                            .unwrap()
                            .parse::<u16>()
                            .unwrap();
                        let authorization = headers.get(&String::from("Authorization")).unwrap();
                        setup_request!(url, c_seq, authorization, port)
                    }
                    RtspMethod::Play => {
                        let session_id = headers.get(&String::from("Session")).unwrap();
                        let authorization = headers.get(&String::from("Authorization")).unwrap();
                        play_request!(url, c_seq, authorization, session_id)
                    }
                    RtspMethod::Teardown => {
                        let session_id = headers.get(&String::from("Session")).unwrap();
                        let authorization = headers.get(&String::from("Authorization")).unwrap();
                        teardown_request!(url, c_seq, authorization, session_id)
                    }
                };
                cursor.write_all(request.as_bytes())?;
                Ok(request.len())
            }
            RtspFrame::RtspResponse { .. } => Err(Error::new(
                ErrorKind::Other,
                "Cannot assemble response frame",
            )),
        }
    }

    pub(crate) fn check_response(buf: &BytesMut) -> Result<()> {
        // Check if the first line is a valid RTSP response
        if &buf[0..9] == b"RTSP/1.0 " {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Invalid RTSP response"))
        }
    }

    pub(crate) async fn parse_response(cursor: &mut Cursor<&[u8]>) -> Result<(Self, usize)> {
        // Status:
        //      RTSP/1.0 <status_code> <reason_phrase>\r\n
        let mut status = String::new();
        cursor.read_line(&mut status).await?;
        let (status_code, reason_phrase) = scanf!(status, "RTSP/1.0 {u16} {String}").unwrap();

        // General header:
        //      C_Seq: <u16>\r\n
        let mut generic_header = String::new();
        cursor.read_line(&mut generic_header).await?;
        let c_seq = scanf!(generic_header, "CSeq: {}", u16).unwrap();

        // Require header:
        //      <header>: <content>\r\n
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
                status_code,
                reason_phrase,
                c_seq,
                headers,
                body,
            },
            cursor.position() as usize,
        ))
    }
}
