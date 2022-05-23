use crate::rtsp_session::RtspSession;
use sscanf::scanf;
use std::collections::HashMap;
use tokio::{
    io::{Error, ErrorKind, Result},
    net::TcpStream,
};

#[derive(Debug)]
pub(crate) struct RtspConnection {
    pub(crate) url: String,
    pub(crate) ipaddr: String,
    pub(crate) rtsp_port: u16,
    pub(crate) rtp_port: u16,
    pub(crate) session_id: String,
    pub(crate) authentication: Option<RtspAuthentication>,
}

#[derive(Debug)]
pub(crate) struct RtspAuthentication {
    pub(crate) user: String,
    pub(crate) passwd: String,
    pub(crate) realm: String,
    pub(crate) nonce: String,
    pub(crate) uri: String,
    pub(crate) response: String,
}

/// A trait for parsing RTSP URL
pub(crate) trait EstablishRtspConnection {
    fn establish_rtsp_connection(&self, url: &String) -> Result<RtspConnection>;
}

#[derive(Debug)]
pub(crate) enum RtspCamera {
    AxisCamera(AxisCamera),
    // Pending
}

impl EstablishRtspConnection for RtspCamera {
    fn establish_rtsp_connection(&self, url: &String) -> Result<RtspConnection> {
        match self {
            RtspCamera::AxisCamera(camera) => camera.establish_rtsp_connection(url),
            #[allow(unreachable_patterns)]
            _ => unimplemented!(),
        }
    }
}

/// the format of Axis camera RTSP URL is:
///     rtsp://root:123456@192.168.1.64/axis-media/media.amp?videocodec=h264&resolution=1280x720&fps=25
#[derive(Debug)]
pub(crate) struct AxisCamera {}

impl EstablishRtspConnection for AxisCamera {
    fn establish_rtsp_connection(&self, url: &String) -> Result<RtspConnection> {
        let (user, passwd, ipaddr, suburl) =
            scanf!(url, "rtsp://{String}:{String}@{String}/{String}")
                .or_else(|err| Err(Error::new(ErrorKind::InvalidInput, format!("{}", err))))?;
        Ok(RtspConnection {
            url: format!("rtsp://{}/{}", ipaddr, suburl),
            ipaddr: ipaddr.clone(),
            rtsp_port: 554,
            rtp_port: 20000,
            session_id: String::new(),
            authentication: Some(RtspAuthentication {
                user,
                passwd,
                realm: String::new(),
                nonce: String::new(),
                uri: format!("rtsp://{}/{}", ipaddr, suburl),
                response: String::new(),
            }),
        })
    }
}

pub(crate) async fn establish_rtsp_connection_and_session(
    args: &HashMap<String, String>,
) -> Result<(RtspConnection, RtspSession)> {
    // Get the camera type
    let camera = match args.get(&String::from("Camera")).unwrap().as_str() {
        "Axis" | "axis" => RtspCamera::AxisCamera(AxisCamera {}),
        _ => unimplemented!(),
    };

    // Parse port number
    let port = args
        .get(&String::from("Port"))
        .unwrap()
        .parse::<u16>()
        .unwrap();

    // Parse the RTSP connection
    let url = args.get(&String::from("Url")).unwrap();
    let mut rtsp_connection = camera.establish_rtsp_connection(&url)?;
    rtsp_connection.rtp_port = port;

    // Establish tcp stream
    let stream = TcpStream::connect(format!("{}:{}", rtsp_connection.ipaddr, port)).await?;

    Ok((rtsp_connection, RtspSession::new(stream)))
}
