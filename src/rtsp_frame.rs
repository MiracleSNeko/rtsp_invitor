use bytes::{BufMut, BytesMut};
use sscanf::scanf;
use std::{collections::HashMap, io::Cursor};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, Error, ErrorKind, Result};

use crate::{
    describe_authenticate_request, describe_request, option_request, play_authenticate_request,
    play_request, setup_authenticate_request, setup_request, teardown_authenticate_request,
    teardown_request,
};

/// Defining an enumeration of the possible methods that can be used in an RTSP request.
#[derive(Debug)]
pub(crate) enum RtspMethod {
    Option,
    Describe,
    Setup,
    Play,
    Teardown,
}

pub(crate) type RtspHeaderMap = HashMap<String, String>;

/// Convert raw rtsp message to data frame
#[derive(Debug)]
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
        _body: String,
    },
}

impl RtspFrame {
    /// The function takes a `RtspFrame` enum and a mutable reference to a `BytesMut` buffer. It then
    /// matches on the enum and assembles the appropriate rtsp request message and writes it to the buffer
    /// 
    /// Arguments:
    /// 
    /// * `buf`: &mut BytesMut, the buffer to write the rtsp request to
    /// 
    /// Returns:
    /// 
    /// The return type is a Result<usize>, which is the number of bytes written to the buffer.
    pub(crate) fn assemble_request(&self, buf: &mut BytesMut) -> Result<usize> {
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
                        if let Some(authorization) = headers.get(&String::from("Authorization")) {
                            setup_authenticate_request!(url, c_seq, authorization, port)
                        } else {
                            setup_request!(url, c_seq, port)
                        }
                    }
                    RtspMethod::Play => {
                        let session_id = headers.get(&String::from("Session")).unwrap();
                        if let Some(authorization) = headers.get(&String::from("Authorization")) {
                            play_authenticate_request!(url, c_seq, authorization, session_id)
                        } else {
                            play_request!(url, c_seq, session_id)
                        }
                    }
                    RtspMethod::Teardown => {
                        let session_id = headers.get(&String::from("Session")).unwrap();
                        if let Some(authorization) = headers.get(&String::from("Authorization")) {
                            teardown_authenticate_request!(url, c_seq, authorization, session_id)
                        } else {
                            teardown_request!(url, c_seq, session_id)
                        }
                    }
                };
                buf.put(request.as_bytes());
                Ok(request.len())
            }
            RtspFrame::RtspResponse { .. } => Err(Error::new(
                ErrorKind::Other,
                "Cannot assemble response frame",
            )),
        }
    }

    /// If the first 9 bytes of the buffer are equal to the string "RTSP/1.0 ", then the function
    /// returns Ok(()), otherwise it returns an error
    /// 
    /// Arguments:
    /// 
    /// * `buf`: &BytesMut, the buffer to read from
    /// 
    /// Returns:
    /// 
    /// A Result<()>, which is Ok(()) if the first 9 bytes of the buffer passed the check
    pub(crate) fn check_response(buf: &BytesMut) -> Result<()> {
        // Check if the first line is a valid RTSP response
        if &buf[0..9] == b"RTSP/1.0 " {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Invalid RTSP response"))
        }
    }

    /// It reads raw rtsp response message from given buffer and parse it into a `RtspFrame` enum
    /// 
    /// Arguments:
    /// 
    /// * `cursor`: &mut Cursor<&[u8]>, the buffer to read from
    /// 
    /// Returns:
    /// 
    /// A tuple of the RtspFrame and the position of the cursor.
    pub(crate) async fn parse_response(cursor: &mut Cursor<&[u8]>) -> Result<(Self, usize)> {
        // Status:
        //      RTSP/1.0 <status_code> <reason_phrase>\r\n
        let mut status = String::new();
        cursor.read_line(&mut status).await?;
        status = String::from(status.trim());
        let (status_code, reason_phrase) = scanf!(status, "RTSP/1.0 {u16} {String}")
            .or_else(|err| Err(Error::new(ErrorKind::Other, err.to_string())))?;

        // General header:
        //      C_Seq: <u16>\r\n
        let mut generic_header = String::new();
        cursor.read_line(&mut generic_header).await?;
        generic_header = String::from(generic_header.trim());
        let c_seq = scanf!(generic_header, "CSeq: {}", u16)
            .or_else(|err| Err(Error::new(ErrorKind::Other, err.to_string())))?;

        // Require header:
        //      <header>: <content>\r\n
        let mut headers = RtspHeaderMap::new();
        loop {
            let mut line = String::new();
            cursor.read_line(&mut line).await?;
            line = String::from(line.trim());

            if line.is_empty() {
                break;
            }

            let mut parts = line.splitn(2, ':');
            let header = parts.next().unwrap().trim();
            let content = parts.next().unwrap().trim();

            headers.insert(header.to_string(), content.to_string());
        }

        // SDP body, ignore it
        let mut body = String::new();
        cursor.read_to_string(&mut body).await?;
        body = String::from(body.trim());

        Ok((
            RtspFrame::RtspResponse {
                status_code,
                reason_phrase,
                c_seq,
                headers,
                _body: body,
            },
            cursor.position() as usize,
        ))
    }
}
