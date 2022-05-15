use sscanf::scanf;
use std::{
    io::{self, BufRead, BufReader, BufWriter},
    net::TcpStream,
};

/// Self defined I/O macros
macro_rules! new_bufio {
    () => {{
        (io::stdin(), io::stdout(), String::new())
    }};
}
macro_rules! init_lockedio {
    ($cin: expr, $cout: expr) => {{
        (BufReader::new($cin.lock()), BufWriter::new($cout.lock()))
    }};
}
macro_rules! getline {
    ($cin: expr, $buf: expr) => {{
        $buf.clear();
        $cin.read_line(&mut $buf)?;
        $buf = $buf.trim().to_string();
    }};
}

/// A parsed RTSP url.
///
/// * `url`: The URL of the RTSP stream
/// * `user`: The username to use for authentication
/// * `passwd`: The password for the camera
/// * `ipaddr`: The IP address of the camera
#[derive(Debug, Clone)]
pub(crate) struct RtspConnection {
    pub(crate) url: String,
    pub(crate) user: String,
    pub(crate) passwd: String,
    pub(crate) ipaddr: String,
}

/// A constructor of RtspConnection struct.
impl RtspConnection {
    /// It takes a string, and returns a struct
    ///
    /// Arguments:
    ///
    /// * `url`: The RTSP URL of the camera.
    ///
    /// Returns:
    ///
    /// A RtspConnection struct
    pub fn from_rtsp_url(url: &str) -> RtspConnection {
        let (user, passwd, ipaddr, uri) =
            scanf!(url, "rtsp://{String}:{String}@{String}/{String}").unwrap();
        RtspConnection {
            url: format!("rtsp://{}/{}", ipaddr, uri),
            user,
            passwd,
            ipaddr,
        }
    }
}

const RTSP_OPTION_REQUEST: &str = "OPTIONS rtsp://{}/{} RTSP/1.0\r\n\
                                   CSeq: {}\r\n\
                                   User-Agent: rtsp-invitor v0.1.0\r\n\
                                   Accept: application/sdp\r\n\
                                   \r\n";
const RTSP_OPTION_RESPONSE: &str = "RTSP/1.0 {} {}\r\n\
                                   CSeq: {}\r\n\
                                   Server: {}\r\n\
                                   Content-Length: {}\r\n\
                                   \r\n{}";

fn main() -> io::Result<()> {
    // Init io streams
    let (cin, cout, mut buf) = new_bufio!();
    let (mut cin_lock, mut cout_lock) = init_lockedio!(cin, cout);

    // Parse input rtsp url
    println!("Please enter the RTSP URL of the camera: (str)");
    getline!(cin_lock, buf);
    let rtsp_connection = RtspConnection::from_rtsp_url(&buf);

    // Get target camera's RTSP port
    println!("Please enter the RTSP port of the camera: (u16, default 554)");
    getline!(cin_lock, buf);
    let rtsp_port = scanf!(buf, "{u16}").unwrap_or(554);

    // Get target RTP port
    println!("Please enter the target RTP port: (u16)");
    getline!(cin_lock, buf);
    let rtp_dst_port = scanf!(buf, "{u16}").unwrap();

    // Establish tcp connection to RTSP server (camera)
    let mut rtsp_client = TcpStream::connect(format!("{}:554", rtsp_connection.ipaddr))?;

    // Step 1: Send OPTIONS request to RTSP server
    let mut cseq = 1;

    // Exit
    Ok(())
}

#[test]
fn test_url_parser() {
    let axis_onvif_url = String::from("rtsp://admin:passw0rd@192.168.3.100/onvif-media/media.amp?profile=profile_1_h264&sessiontimeout=60&streamtype=unicast");
    let rtsp_connection = RtspConnection::from_rtsp_url(&axis_onvif_url);

    assert_eq!(rtsp_connection.user, String::from("admin"));
    assert_eq!(rtsp_connection.passwd, String::from("passw0rd"));
    assert_eq!(rtsp_connection.ipaddr, String::from("192.168.3.100"));
}
