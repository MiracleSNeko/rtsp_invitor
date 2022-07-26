use crate::rtsp_session::RtspSession;
use sscanf::scanf;
use std::collections::HashMap;
use tokio::{
    io::{Error, ErrorKind, Result},
    net::TcpStream,
};

/// `RtspConnection` is a struct that contains a `String` called `url`, a `String` called `ipaddr`, a
/// `u16` called `rtsp_port`, a `u16` called `rtp_port`, a `String` called `session_id`, and an
/// `Option<RtspAuthentication>` called `authentication`.
/// 
/// Properties:
/// 
/// * `url`: The URL of the RTSP stream.
/// * `ipaddr`: The IP address of the camera
/// * `rtsp_port`: The port that the RTSP server is listening on.
/// * `rtp_port`: The port that the RTP data will be sent to.
/// * `session_id`: The session ID is a unique identifier for the RTSP session.
/// * `authentication`: Option<RtspAuthentication>
#[derive(Debug)]
pub(crate) struct RtspConnection {
    pub(crate) url: String,
    pub(crate) ipaddr: String,
    pub(crate) rtsp_port: u16,
    pub(crate) rtp_port: u16,
    pub(crate) session_id: String,
    pub(crate) authentication: Option<RtspAuthentication>,
}

/// `RtspAuthentication` is a struct that contains a `user`, `passwd`, `realm`, `nonce`, and `uri`
/// field.
/// 
/// The `pub(crate)` keyword means that the struct is public within the crate, but not outside of it.
/// 
/// The `Debug` trait is a built-in trait that allows the struct to be printed to the console.
/// 
/// The `String` type is a string type that is allocated on the heap.
/// 
/// The `user`, `passwd`, `realm`,
/// 
/// Properties:
/// 
/// * `user`: The user name
/// * `passwd`: The password for the user.
/// * `realm`: The realm is a string that defines the protection space. If a server wishes to limit the
/// access to only a portion of the server, it may indicate that with the realm value.
/// * `nonce`: A server-specified data string which should be uniquely generated each time a 401
/// response is made. It is recommended that this string be base64 or hexadecimal data. Specifically,
/// since the string is passed in the header lines as a quoted string, the double-quote character is not
/// allowed.
/// * `uri`: The URI of the resource being requested.
#[derive(Debug)]
pub(crate) struct RtspAuthentication {
    pub(crate) user: String,
    pub(crate) passwd: String,
    pub(crate) realm: String,
    pub(crate) nonce: String,
    pub(crate) uri: String,
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
///     rtsp://root:ms10+njrjdd50H@10.229.86.28/axis-media/media.amp?videocodec=h264&resolution=1280x720&fps=25
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
            }),
        })
    }
}

/// It takes a HashMap of arguments, and returns a tuple of an RtspConnection and an RtspSession
/// 
/// Arguments:
/// 
/// * `args`: &HashMap<String, String>
/// 
/// Returns:
/// 
/// A tuple of RtspConnection and RtspSession
pub(crate) async fn establish_rtsp_connection_and_session(
    args: &HashMap<String, String>,
) -> Result<(RtspConnection, RtspSession)> {
    // Get the camera type
    let camera = match args.get(&String::from("Camera")).unwrap().as_str() {
        "Axis" | "axis" => RtspCamera::AxisCamera(AxisCamera {}),
        _ => unimplemented!(),
    };

    // Parse port number
    let ports = args.get(&String::from("Port")).unwrap();
    let (rtsp_port, rtp_port) = scanf!(ports, "{u16}:{u16}").unwrap();

    // Parse the RTSP connection
    let url = args.get(&String::from("Url")).unwrap();
    let mut rtsp_connection = camera.establish_rtsp_connection(&url)?;
    rtsp_connection.rtp_port = rtp_port;
    rtsp_connection.rtsp_port = rtsp_port;

    // Establish tcp stream
    let stream = TcpStream::connect(format!(
        "{}:{}",
        rtsp_connection.ipaddr, rtsp_connection.rtsp_port
    ))
    .await?;

    Ok((rtsp_connection, RtspSession::new(stream)))
}
