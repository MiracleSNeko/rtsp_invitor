use crate::{
    rtsp_machine::RtspState,
    rtsp_url_parser::{AxisCamera, EstablishRtspConnection, RtspCamera},
};
use sscanf::scanf;
use std::env;
use std::io::{BufRead, BufReader, BufWriter, Write};
use tokio::{
    io::{Interest, Result},
    net::TcpStream,
};

pub(crate) mod io_macros;
pub(crate) mod rtsp_frame;
pub(crate) mod rtsp_machine;
pub(crate) mod rtsp_request;
pub(crate) mod rtsp_response;
pub(crate) mod rtsp_session;
pub(crate) mod rtsp_url_parser;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // Init io streams
    #[allow(unused_assignments)]
    let (cin, cout, mut buf) = new_bufio!();
    let (mut cin_lock, _) = init_lockedio!(cin, cout);

    // Get input
    let args = env::args().skip(1).collect::<Vec<_>>();
    assert_eq!(
        args.len(),
        4,
        "Invalid input args! Usage: rtsp_invitor -i <rtsp_url> -o <ip.dst:port>."
    );

    // Parse input
    args.as_slice()
        .windows(2)
        .for_each(|input| match input[0].as_str() {
            "-i" => {}
            "-o" => {}
            _ => panic!("Invalid input args! Usage: rtsp_invitor -i <rtsp_url> -o <ip.dst:port>."),
        });

    // Get camera type
    // println!("Please input the camera type: (str, default Axis)");
    // buf = getline!(cin_lock)?;
    // let camera_type = scanf!(buf, "{String}").unwrap_or(String::from("Axis"));
    // let camera = match camera_type.as_str() {
    //     "Axis" => RtspCamera::AxisCamera(AxisCamera {}),
    //     _ => panic!("Unsupported camera type!"),
    // };

    // // Parse input rtsp url
    // println!("Please enter the RTSP URL of the camera: (str)");
    // buf = getline!(cin_lock)?;
    // let mut rtsp_connection = match camera {
    //     RtspCamera::AxisCamera(axis_camera) => axis_camera.establish_rtsp_connection(buf)?,
    //     #[allow(unreachable_patterns)]
    //     _ => panic!("Unsupported camera type!"),
    // };

    // // Get target camera's RTSP port
    // println!("Please enter the RTSP port of the camera: (u16, default 554)");
    // buf = getline!(cin_lock)?;
    // let rtsp_port = scanf!(buf, "{u16}").unwrap_or(554);
    // rtsp_connection.port = rtsp_port;

    // // Assembly RTSP machine
    // let mut rtsp_state = RtspState::Starting;

    // // Get target RTP port
    // println!("Please enter the target RTP port: (u16)");
    // buf = getline!(cin_lock)?;
    // let rtp_dst_port = scanf!(buf, "{u16}").unwrap();

    // // Establish tcp connection to RTSP server (camera)
    // println!("Establishing RTSP connection...");
    // let tcp_stream = TcpStream::connect(format!(
    //     "{}:{}",
    //     rtsp_connection.ipaddr, rtsp_connection.port
    // ))
    // .await?;

    // // Send RTSP request and receive RTSP response
    // let mut c_seq = 1;
    // loop {
    //     let ready = tcp_stream
    //         .ready(Interest::WRITABLE | Interest::READABLE)
    //         .await?;

    //     // Send RTSP request
    //     if ready.is_writable() {}

    //     // Recv RTSP response
    //     if ready.is_readable() {}
    // }

    // // waiting for console input to stop RTP stream
    // println!("Enter \"exit\" to stop RTP stream...");
    // loop {
    //     buf = getline!(cin_lock)?;
    //     if buf == "exit" {
    //         break;
    //     } else {
    //         println!("Enter \"exit\" to stop RTP stream...");
    //         continue;
    //     }
    // }

    // // Send TEARDOWN request and receive TEARDOWN response
    // loop {
    //     let ready = tcp_stream
    //         .ready(Interest::WRITABLE | Interest::READABLE)
    //         .await?;

    //     // Send TEARDOWN request
    //     if ready.is_writable() && rtsp_state == RtspState::PlayRecv {
    //         let request = teardown_request!(rtsp_connection.url, c_seq, rtsp_connection.session_id);
    //         println!("Sending request: {}", request);
    //         match tcp_stream.try_write(request.as_bytes()) {
    //             Ok(_) => {
    //                 rtsp_state = RtspState::Exiting;
    //             }
    //             Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    //                 continue;
    //             }
    //             Err(ref e) => {
    //                 return Err(io::Error::new(e.kind(), format!("{}", e)));
    //             }
    //         }
    //     }

    //     // Recv TEARDOWN response
    //     if ready.is_readable() && rtsp_state == RtspState::Exiting {
    //         let mut buff = [0_u8; 1500]; // pre-allocate 1500 bytes (MTU of ethernet)
    //         match tcp_stream.try_read(&mut buff) {
    //             Ok(len) => {
    //                 println!(
    //                     "Received response: {}",
    //                     String::from_utf8_lossy(&buff[..len])
    //                 );
    //                 break;
    //             }
    //             Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    //                 continue;
    //             }
    //             Err(ref e) => {
    //                 return Err(io::Error::new(e.kind(), format!("{}", e)));
    //             }
    //         }
    //     }
    // }

    // Exit
    Ok(())
}
