use crate::rtsp_camera::{establish_rtsp_connection_and_session, RtspConnection};
use crate::rtsp_frame::{
    RtspFrame::{self, RtspRequest},
    RtspHeaderMap, RtspMethod,
};
use crate::rtsp_session::RtspSession;
use md5::compute as md5;
use sscanf::scanf;
use std::collections::HashMap;
use tokio::io::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RtspState {
    Option,
    Describe,
    Authenticate,
    Setup,
    Play,
    Teardown,
}

#[derive(Debug)]
pub(crate) struct RtspMachine {
    rtsp_session: RtspSession,
    rtsp_connection: RtspConnection,
    rtsp_state: RtspState,
}

impl RtspMachine {
    pub(crate) async fn new(args: &HashMap<String, String>) -> Result<Self> {
        let (connection, session) = establish_rtsp_connection_and_session(args).await?;
        Ok(Self {
            rtsp_session: session,
            rtsp_connection: connection,
            rtsp_state: RtspState::Option,
        })
    }

    /// - Return Ok(true) when OPTHIN/DESCRIBE/SETUP/PLAY request sended
    /// - Return Ok(false) when try to send TEARDOWN request
    /// - Return Err(err) in error case
    pub(crate) async fn process_request(&mut self, c_seq: u16) -> Result<bool> {
        let frame = match self.rtsp_state {
            RtspState::Option => Ok(Some(RtspRequest {
                method: RtspMethod::Option,
                url: self.rtsp_connection.url.clone(),
                c_seq,
                headers: HashMap::new(),
            })),
            RtspState::Describe => Ok(Some(RtspRequest {
                method: RtspMethod::Describe,
                url: self.rtsp_connection.url.clone(),
                c_seq,
                headers: HashMap::new(),
            })),
            RtspState::Authenticate => {
                let auth = self.authenticate().map_or_else(
                    || {
                        Err(Error::new(
                            ErrorKind::Other,
                            "Authentication info not founded!",
                        ))
                    },
                    |auth| Ok(auth),
                )?;
                let mut headers = HashMap::new();
                headers.insert(String::from("Authorization"), auth);
                Ok(Some(RtspRequest {
                    method: RtspMethod::Describe,
                    url: self.rtsp_connection.url.clone(),
                    c_seq,
                    headers,
                }))
            }
            RtspState::Setup => {
                let mut headers = HashMap::new();
                if let Some(auth) = self.authenticate() {
                    headers.insert(String::from("Authorization"), auth);
                };
                headers.insert(
                    String::from("Port"),
                    self.rtsp_connection.rtp_port.to_string(),
                );
                Ok(Some(RtspRequest {
                    method: RtspMethod::Setup,
                    url: self.rtsp_connection.url.clone(),
                    c_seq,
                    headers,
                }))
            }
            RtspState::Play => {
                let mut headers = HashMap::new();
                if let Some(auth) = self.authenticate() {
                    headers.insert(String::from("Authorization"), auth);
                };
                headers.insert(
                    String::from("Session"),
                    self.rtsp_connection.session_id.clone(),
                );
                Ok(Some(RtspRequest {
                    method: RtspMethod::Play,
                    url: self.rtsp_connection.url.clone(),
                    c_seq,
                    headers,
                }))
            }
            RtspState::Teardown => Ok(None),
        };
        // Send Rtsp request
        match frame {
            Ok(frame) => match frame {
                Some(frame) => {
                    self.rtsp_session.write_frame(&frame).await?;
                    Ok(true)
                }
                None => Ok(false),
            },
            Err(err) => Err(err),
        }
    }

    pub(crate) async fn process_response(&mut self, c_seq: u16) -> Result<u16> {
        let frame = self.rtsp_session.read_frame().await?;
        if let Some(ref frame) = frame {
            let (headers, status_code) = self.get_response_headers(frame, c_seq)?;
            match self.rtsp_state {
                RtspState::Option => {
                    self.rtsp_state = RtspState::Describe;
                }
                RtspState::Describe => match status_code {
                    401 => {
                        // Describe failed, need authentication
                        let auth = self.rtsp_connection.authentication.as_mut().unwrap();
                        let buffer = headers.get(&String::from("WWW-Authenticate")).unwrap();
                        let (realm, nonce, _) = scanf!(
                            buffer,
                            "Digest realm=\"{String}\", nonce=\"{String}\",{String}"
                        )
                        .unwrap();
                        auth.realm = realm;
                        auth.nonce = nonce;
                        self.rtsp_state = RtspState::Authenticate;
                    }
                    200 => {
                        // Describe succeed, do not need authentication
                        self.rtsp_connection.authentication = None;
                        self.rtsp_state = RtspState::Setup;
                    }
                    _ => unreachable!(),
                },
                RtspState::Authenticate => {
                    // Ignore sdp
                    self.rtsp_state = RtspState::Setup;
                }
                RtspState::Setup => {
                    let buffer = headers.get(&String::from("Session")).unwrap();
                    let (session_id, _) = scanf!(buffer, "{String};{String}").unwrap();
                    self.rtsp_connection.session_id = session_id;
                    self.rtsp_state = RtspState::Play;
                }
                RtspState::Play => {
                    self.rtsp_state = RtspState::Teardown;
                }
                RtspState::Teardown => {}
            }
            Ok(c_seq + 1)
        } else {
            // Need re-recv Rtsp response
            Ok(0)
        }
    }

    pub(crate) async fn shut_down(&mut self, c_seq: u16) -> Result<usize> {
        let mut headers = HashMap::new();
        if let Some(auth) = self.authenticate() {
            headers.insert(String::from("Authorization"), auth);
        };
        headers.insert(
            String::from("Session"),
            self.rtsp_connection.session_id.clone(),
        );
        let frame = RtspRequest {
            method: RtspMethod::Teardown,
            url: self.rtsp_connection.url.clone(),
            c_seq,
            headers,
        };
        self.rtsp_session.write_frame(&frame).await
    }

    pub(crate) fn clear_buf(&mut self) {
        self.rtsp_session.buf.clear()
    }

    fn authenticate(&self) -> Option<String> {
        if let Some(ref auth) = self.rtsp_connection.authentication {
            let method = match self.rtsp_state {
                RtspState::Option => "OPTION",
                RtspState::Describe => "DESCRIBE",
                RtspState::Authenticate => "DESCRIBE",
                RtspState::Setup => "SETUP",
                RtspState::Play => "PLAY",
                RtspState::Teardown => "TEARDOWN",
            };
            let response = format!(
                "{:x}",
                md5(format!(
                    "{:x}:{}:{:x}",
                    md5(format!("{}:{}:{}", auth.user, auth.realm, auth.passwd)),
                    auth.nonce,
                    md5(format!("{}:{}", method, auth.uri))
                ))
            );
            Some(format!(
                "Digest username=\"{0}\", realm=\"{1}\", nonse=\"{2}\", uri=\"{3}\", response=\"{4}\"",
                auth.user, auth.realm, auth.nonce, auth.uri, response
            ))
        } else {
            None
        }
    }

    fn get_response_headers(&self, frame: &RtspFrame, c_seq: u16) -> Result<(RtspHeaderMap, u16)> {
        match frame {
            RtspFrame::RtspResponse {
                status_code,
                reason_phrase,
                c_seq: c_seq_real,
                headers,
                ..
            } => {
                if *status_code == 200
                    || (*status_code == 401 && self.rtsp_state == RtspState::Describe)
                {
                    if c_seq != *c_seq_real {
                        Err(Error::new(
                            ErrorKind::Other,
                            format!("CSeq mismatch: expected {}, get {}.", c_seq, c_seq_real),
                        ))
                    } else {
                        Ok((headers.clone(), *status_code))
                    }
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        format!(
                            "Rtsp response error: RTSP/1.0 {} {}.",
                            status_code, reason_phrase
                        ),
                    ))
                }
            }
            RtspFrame::RtspRequest { .. } => {
                Err(Error::new(ErrorKind::Other, "Cannot parse request frame."))
            }
        }
    }
}

#[test]
fn test_digest_authentication() {
    let response = format!(
        "{:x}",
        md5(format!(
            "{:x}:{}:{:x}",
            md5(format!(
                "{}:{}:{}",
                "admin", "AXIS_WS_ACCC8EE2525A", "ms10+njrjdd50H"
            )),
            "00000140Y557448441a443427fbfb9b19ef531ca05ac078",
            md5(format!(
                "{}:{}",
                "DESCRIBE", "rtsp://10.229.86.28:554/onvif-media/media.amp"
            ))
        ))
    );
    assert_eq!(response.as_str(), "1a8d0b61fb45d29200791fd10238aea4");
}
