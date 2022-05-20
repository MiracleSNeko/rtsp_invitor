use sscanf::scanf;

/// OPTIONS response format:
///
/// ```bash
/// RTSP/1.0 200 OK\r\n
/// CSeq: 2\r\n
/// Public: OPTIONS, DESCRIBE, ANNOUNCE, GET_PARAMETER, PAUSE, PLAY, RECORD, SETUP, SET_PARAMETER, TEARDOWN\r\n
/// Server: GStreamer RTSP server\r\n
/// Date: Wed, 18 May 2022 11:14:01 GMT\r\n
/// \r\n
/// ```
///
/// this macro will return (status, c_seq, public_methods, _)
#[macro_export]
macro_rules! option_response {
    ($buf: expr) => {{
        println!("Response to OPTION: \r\n{}", $buf);
        scanf!(
            $buf,
            "RTSP/1.0 {String}\r\nCSeq: {u16}\r\nPublic: {String}\r\n{String}"
        )
    }};
}

#[test]
fn test_option_response() {
    let response = "RTSP/1.0 200 OK\r\nCSeq: 2\r\nPublic: OPTIONS, DESCRIBE, ANNOUNCE, GET_PARAMETER, PAUSE, PLAY, RECORD, SETUP, SET_PARAMETER, TEARDOWN\r\nServer: GStreamer RTSP server\r\nDate: Wed, 18 May 2022 11:14:01 GMT\r\n\r\n";
    let (status, c_seq, public_methods, _) = option_response!(response).unwrap();
    println!(
        "status: {}, c_seq: {}, public_methods: {}",
        status, c_seq, public_methods
    );
}

/// DESCRIBE response format:
///
/// ```bash
/// RTSP/1.0 200 OK\r\n
/// CSeq: 3\r\n
/// Content-type: application/sdp
/// Content-Base: rtsp://10.229.86.28:554/onvif-media/media.amp/\r\n
/// Server: GStreamer RTSP server\r\n
/// Date: Wed, 18 May 2022 11:13:06 GMT\r\n
/// Content-length: 776
/// \r\n
/// Session Description Protocol Version (v): 0
/// Owner/Creator, Session Id (o): - 2786656593419956836 1 IN IP4 10.229.86.28
/// Session Name (s): Session streamed with GStreamer
/// Session Information (i): rtsp-server
/// Time Description, active time (t): 0 0
/// Session Attribute (a): tool:GStreamer
/// Session Attribute (a): type:broadcast
/// Session Attribute (a): range:npt=now-
/// Session Attribute (a): control:rtsp://10.229.86.28:554/onvif-media/media.amp?profile=profile_1_h264&sessiontimeout=60&streamtype=unicast
/// Media Description, name and address (m): video 0 RTP/AVP 96
/// Connection Information (c): IN IP4 0.0.0.0
/// Bandwidth Information (b): AS:50000
/// Media Attribute (a): rtpmap:96 H264/90000
/// Media Attribute (a): fmtp:96 packetization-mode=1;profile-level-id=4d0029;sprop-parameter-sets=Z00AKeKQDwBE/LgLcBAQGlAAbd0AGb/MAPEiKg==,aO48gA==
/// Media Attribute (a): ts-refclk:local
/// Media Attribute (a): mediaclk:sender
/// Media Attribute (a): recvonly
/// Media Attribute (a): control:rtsp://10.229.86.28:554/onvif-media/media.amp/stream=0?profile=profile_1_h264&sessiontimeout=60&streamtype=unicast
/// Media Attribute (a): framerate:30.000000
/// Media Attribute (a): transform:1.000000,0.000000,0.000000;0.000000,1.000000,0.000000;0.000000,0.000000,1.000000
/// \r\n
/// ```
///
/// or
///
///```bash
/// RTSP/1.0 401 Unauthorized\r\n
/// CSeq: 3\r\n
/// WWW-Authenticate: Digest realm="AXIS_WS_ACCC8EE2525A", nonce="0000015aY178238f227956fc1b41b45c3c8320c230c744c", stale=FALSE\r\n
/// Server: GStreamer RTSP server\r\n
/// Date: Wed, 18 May 2022 11:14:01 GMT\r\n
/// \r\n
/// ```
/// this macro will return (status, c_seq, _)
/// 
/// 
/// # todo:
/// 1. parse input buf by lines
/// 2. DO NOT use String as buf, use BufReader instead
#[macro_export]
macro_rules! describe_response {
    ($buf: expr) => {{
        println!("Response to DESCRIBE: \r\n{}", $buf);
        scanf!($buf, "RTSP/1.0 {String}\r\nCSeq: {u16}\r\n{String}")
    }};
}
/// this macro will return (c_seq, realm, nonce, stale)
#[macro_export]
macro_rules! describe_unauthorized_response {
    ($buf: expr) => {{
        println!("Response to DESCRIBE: \r\n{}", $buf);
        scanf!(
            $buf,
            "RTSP/1.0 401 Unauthorized\r\nCSeq: {u16}\r\n
             WWW-Authenticate: Digest realm={String}, nonce={String}, stale={String}\r\n
             {String}"
        )
    }};
}
/// ignore sdp

/// SETUP response format:
///
/// ```bash
/// RTSP/1.0 200 OK\r\n
/// CSeq: 5\r\n
/// Transport: RTP/AVP;unicast;client_port=51052-51053;server_port=50000-50001;ssrc=425E85AB;mode="PLAY"
/// Server: GStreamer RTSP server\r\n
/// Session: xdkiTkdbOUYVuz2u;timeout=60
/// Date: Wed, 18 May 2022 11:13:06 GMT\r\n
/// \r\n
/// ```
///
/// this macro will return (status, c_seq, _, session_id, _)
#[macro_export]
macro_rules! setup_response {
    ($buf: expr) => {{
        println!("Response to SETUP: \r\n{}", $buf);
        scanf!(
            $buf,
            "RTSP/1.0 {String}\r\nCSeq: {u16}\r\n
             {String}\r\n
             Session: {String};{String}"
        )
    }};
}

/// PLAY response format:
///
/// ```bash
/// RTSP/1.0 200 OK\r\n
/// CSeq: 6\r\n
/// RTP-Info: url=rtsp:://10.229.86.28:554/onvif-media/media.amp/stream=0?profile=profile_1_h264&sessiontimeout=60&streamtype=unicast;seq=4409;rtptime=1173479798\r\n
/// Range: npt=now-\r\n
/// Server: GStreamer RTSP server\r\n
/// Session: xdkiTkdbOUYVuz2u;timeout=60
/// Date: Wed, 18 May 2022 11:13:06 GMT\r\n
/// \r\n
/// ```
///
/// this macro will return (status, c_seq, _,)
#[macro_export]
macro_rules! play_response {
    ($buf: expr) => {{
        println!("Response to PLAY: \r\n{}", $buf);
        scanf!($buf, "RTSP/1.0 {String}\r\nCSeq: {u16}\r\n{String}")
    }};
}

/// TEARDOWN response format:
///
/// ```bash
/// RTSP/1.0 200 OK\r\n
/// CSeq: 8\r\n
/// Server: GStreamer RTSP server\r\n
/// Session: xdkiTkdbOUYVuz2u;timeout=60
/// Connection: close\r\n
/// Date: Wed, 18 May 2022 11:13:34 GMT\r\n
/// \r\n
/// ```
///
/// this macro will return (status, c_seq, _)
#[macro_export]
macro_rules! teardown_response {
    ($buf: expr) => {{
        println!("Response to TEARDOWN: \r\n{}", $buf);
        scanf!($buf, "RTSP/1.0 {String}\r\nCSeq: {u16}\r\n{String}")
    }};
}
