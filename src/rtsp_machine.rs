use crate::{describe_authenticate_request, describe_request, rtsp_url_parser::RtspConnection};
use md5::compute as md5;

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
    pub(crate) rtsp_connection: RtspConnection,
    pub(crate) rtsp_state: RtspState,
}

impl RtspMachine {
    pub fn process_request(&mut self, c_seq: u16) -> String {
        match self.rtsp_state {
            RtspState::Describe => {
                describe_request!(self.rtsp_connection.url, c_seq)
            }
            RtspState::Authenticate => {
                if let Some(ref mut auth) = self.rtsp_connection.authentication {
                    auth.response = format!(
                        "{:x}",
                        md5(format!(
                            "{:x}:{}:{:x}",
                            md5(format!("{}:{}:{}", auth.user, auth.realm, auth.passwd)),
                            auth.nonce,
                            md5(format!(
                                "{}:{}",
                                self.rtsp_connection.public_methods, auth.uri
                            ))
                        ))
                    );

                    describe_authenticate_request!(
                        self.rtsp_connection.url,
                        c_seq,
                        auth.user,
                        auth.realm,
                        auth.nonce,
                        auth.response
                    )
                } else {
                    panic!("Authentication information not found, please check the URL!");
                }
            }
            RtspState::Setup => {
                unimplemented!()
            }
            RtspState::Play => {
                unimplemented!()
            }
            RtspState::Teardown => {
                unimplemented!()
            }
            _ => panic!("Unknown RTSP state"),
        }
    }

    pub fn process_response(&mut self, c_seq: u16) {}
}

#[test]
fn test_authentication() {
    let response = format!(
        "{:x}",
        md5(format!(
            "{:x}:{}:{:x}",
            md5(format!("{}:{}:{}", "admin", "AXIS_WS_ACCC8EE2525A", "ms10+njrjdd50H")),
            "0000015aY178238f227956fc1b41b45c3c8320c230c744c",
            md5(format!(
                "{}:{}",
                "OPTIONS, DESCRIBE, ANNOUNCE, GET_PARAMETER, PAUSE, PLAY, RECORD, SETUP, SET_PARAMETER, TEARDOWN", "rtsp://10.229.86.28:554/onvif-media/media.amp"
            ))
        )));
    println!("{}", response);
}
