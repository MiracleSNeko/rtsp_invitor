use sscanf::scanf;
use std::io;

/// `RtspConnection` is a struct that contains a `String` named `url`, a `String` named `ipaddr`, and an
/// `Option<RtspAuthentication>` named `authentication`.
///
/// Properties:
///
/// * `url`: The URL of the RTSP stream.
/// * `ipaddr`: The IP address of the camera.
/// * `port`: The RTSP port of the camera.
/// * `authentication`: The authentication information of the camera.
#[derive(Debug)]
pub(crate) struct RtspConnection {
    pub(crate) url: String,
    pub(crate) ipaddr: String,
    pub(crate) port: u16,
    pub(crate) public_methods: String,
    pub(crate) session_id: String,
    pub(crate) authentication: Option<RtspAuthentication>,
}

/// `RtspAuthentication` is a struct with 6 fields, all of which are public, and all of which are
/// strings.
///
/// Properties:
///
/// * `user`: The username to use for authentication.
/// * `passwd`: The password for the user.
/// * `realm`: The realm is a string that defines the protection space. If a server wishes to limit
/// access to only a portion of the server, it may do so by indicating that portion of the server in the
/// realm portion of the challenge.
/// * `nonce`: A server-specified data string which should be uniquely generated each time a 401
/// response is made. It is recommended that this string be base64 or hexadecimal data. Specifically,
/// since the string is passed in the header lines as a quoted string, the double-quote character is not
/// allowed.
/// * `uri`: The URI of the resource being requested.
/// * `response`: The response is the MD5 hash of the combined nonce, username, password, and realm.
#[derive(Debug)]
pub(crate) struct RtspAuthentication {
    pub(crate) user: String,
    pub(crate) passwd: String,
    pub(crate) realm: String,
    pub(crate) nonce: String,
    pub(crate) uri: String,
    pub(crate) response: String,
}

#[derive(Debug)]
pub(crate) enum RtspCamera {
    AxisCamera(AxisCamera),
    // Pending
}

/// A trait for parsing RTSP URL
pub(crate) trait EstablishRtspConnection {
    fn establish_rtsp_connection(&self, url: String) -> Result<RtspConnection, io::Error>;
}

/// the format of Axis camera RTSP URL is:
///     rtsp://root:123456@192.168.1.64/axis-media/media.amp?videocodec=h264&resolution=1280x720&fps=25
#[derive(Debug)]
pub(crate) struct AxisCamera {}

impl EstablishRtspConnection for AxisCamera {
    fn establish_rtsp_connection(&self, url: String) -> Result<RtspConnection, io::Error> {
        match scanf!(url, "rtsp://{String}:{String}@{String}/{String}") {
            Ok((user, passwd, ipaddr, url)) => Ok(RtspConnection {
                url,
                ipaddr,
                port: 554,
                session_id: String::new(),
                public_methods: String::new(),
                authentication: Some(RtspAuthentication {
                    user,
                    passwd,
                    realm: String::new(),
                    nonce: String::new(),
                    uri: String::new(),
                    response: String::new(),
                }),
            }),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidInput, e.to_string())),
        }
    }
}
