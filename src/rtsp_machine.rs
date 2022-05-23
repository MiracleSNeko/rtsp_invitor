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
    pub(crate) rtsp_session: RtspSession,
    pub(crate) rtsp_connection: RtspConnection,
    pub(crate) rtsp_state: RtspState,
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

    pub(crate) async fn process_request(&mut self, c_seq: u16) -> Result<()> {
        let frame = match self.rtsp_state {
            RtspState::Option => Ok(RtspRequest {
                method: RtspMethod::Option,
                url: self.rtsp_connection.url.clone(),
                c_seq,
                headers: HashMap::new(),
            }),
            RtspState::Describe => Ok(RtspRequest {
                method: RtspMethod::Describe,
                url: self.rtsp_connection.url.clone(),
                c_seq,
                headers: HashMap::new(),
            }),
            RtspState::Authenticate => {
                let auth = self.authenticate()?;
                let mut headers = HashMap::new();
                headers.insert(String::from("Authorization"), auth);
                Ok(RtspRequest {
                    method: RtspMethod::Describe,
                    url: self.rtsp_connection.url.clone(),
                    c_seq,
                    headers,
                })
            }
            RtspState::Setup => {
                let auth = self.authenticate()?;
                let mut headers = HashMap::new();
                headers.insert(String::from("Authorization"), auth);
                headers.insert(
                    String::from("Port"),
                    self.rtsp_connection.rtp_port.to_string(),
                );
                Ok(RtspRequest {
                    method: RtspMethod::Setup,
                    url: self.rtsp_connection.url.clone(),
                    c_seq,
                    headers,
                })
            }
            RtspState::Play => {
                let auth = self.authenticate()?;
                let mut headers = HashMap::new();
                headers.insert(String::from("Authorization"), auth);
                headers.insert(
                    String::from("Session"),
                    self.rtsp_connection.session_id.clone(),
                );
                Ok(RtspRequest {
                    method: RtspMethod::Play,
                    url: self.rtsp_connection.url.clone(),
                    c_seq,
                    headers,
                })
            }
            RtspState::Teardown => Err(Error::new(
                ErrorKind::Other,
                "Cannot send TEARDOWN request in `process_request`",
            )),
        };
        // Send Rtsp request
        match frame {
            Ok(frame) => self.rtsp_session.write_frame(&frame).await,
            Err(err) => Err(err),
        }
    }

    pub(crate) async fn process_response(&mut self, c_seq: u16) -> Result<u16> {
        let frame = self.rtsp_session.read_frame().await?;
        if let Some(ref frame) = frame {
            let headers = self.get_response_headers(frame, c_seq)?;
            match self.rtsp_state {
                RtspState::Option => {
                    self.rtsp_state = RtspState::Describe;
                }
                RtspState::Describe => {
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
                RtspState::Authenticate => {
                    // Ignore sdp
                    self.rtsp_state = RtspState::Setup;
                }
                RtspState::Setup => {
                    let buffer = headers.get(&String::from("Session")).unwrap();
                    let (session_id, _) = scanf!(buffer, "{String}:{String}").unwrap();
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

    pub(crate) async fn shut_down(&mut self, c_seq: u16) -> Result<()> {
        let auth = self.authenticate()?;
        let mut headers = HashMap::new();
        headers.insert(String::from("Authorization"), auth);
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

    fn authenticate(&self) -> Result<String> {
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
            Ok(format!(
                "Digest username=\"{0}\", realm=\"{1}\", nonse=\"{2}\", uri=\"{3}\", response=\"{4}\"",
                auth.user, auth.realm, auth.nonce, auth.uri, response
            ))
        } else {
            Err(Error::new(ErrorKind::Other, "Authentication not found"))
        }
    }

    fn get_response_headers(&self, frame: &RtspFrame, c_seq: u16) -> Result<RtspHeaderMap> {
        let accept_code = if self.rtsp_state == RtspState::Describe {
            401
        } else {
            200
        } as u16;
        match frame {
            RtspFrame::RtspResponse {
                status_code,
                reason_phrase,
                c_seq: c_seq_real,
                headers,
                ..
            } => {
                if *status_code == accept_code {
                    if c_seq != *c_seq_real {
                        Err(Error::new(
                            ErrorKind::Other,
                            format!("CSeq mismatch: expected {}, get {}.", c_seq, c_seq_real),
                        ))
                    } else {
                        Ok(headers.clone())
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
