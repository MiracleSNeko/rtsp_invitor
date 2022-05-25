#[macro_export]
macro_rules! option_request {
    ($url: expr, $c_seq: expr) => {
        format!(
            "OPTIONS {} RTSP/1.0\r\nCSeq: {}\r\nUser-Agent: rtsp-invitor-1.0\r\n\r\n",
            $url, $c_seq
        )
    };
}

#[macro_export]
macro_rules! describe_request {
    ($url: expr, $c_seq: expr) => {
        format!(
            "DESCRIBE {} RTSP/1.0\r\nCSeq: {}\r\nUser-Agent: rtsp-invitor-1.0\r\n\r\n",
            $url, $c_seq
        )
    };
}

#[macro_export]
macro_rules! describe_authenticate_request {
    ($url: expr, $c_seq: expr, $authorization: expr) => {
        format!(
            "DESCRIBE {} RTSP/1.0\r\nCSeq: {}\r\nAuthorization: {}\r\nUser-Agent: rtsp-invitor-1.0\r\n",
            $url, $c_seq, $authorization
        )
    };
}

#[macro_export]
macro_rules! setup_request {
    ($url: expr, $c_seq: expr, $port: expr) => {
        format!(
            "SETUP {} RTSP/1.0\r\nCSeq: {}\r\nUser-Agent: rtsp-invitor-1.0\r\nTransport: RTP/AVP;unicast;client_port={}-{}\r\n\r\n",
            $url,
            $c_seq,
            $port,
            $port + 1
        )
    };
}

#[macro_export]
macro_rules! setup_authenticate_request {
    ($url: expr, $c_seq: expr, $authorization: expr, $port: expr) => {
        format!(
            "SETUP {} RTSP/1.0\r\nCSeq: {}\r\nAuthorization: {}\r\nUser-Agent: rtsp-invitor-1.0\r\nTransport: RTP/AVP;unicast;client_port={}-{}\r\n\r\n",
            $url,
            $c_seq,
            $authorization,
            $port,
            $port + 1
        )
    };
}

#[macro_export]
macro_rules! play_request {
    ($url: expr, $c_seq: expr, $session_id: expr) => {
        format!(
            "PLAY {} RTSP/1.0\r\nCSeq: {}\r\nUser-Agent: rtsp-invitor-1.0\r\nRange: npt=0-\r\nSession: {}\r\n\r\n",
            $url, $c_seq, $session_id
        )
    };
}

#[macro_export]
macro_rules! play_authenticate_request {
    ($url: expr, $c_seq: expr, $authorization: expr, $session_id: expr) => {
        format!(
            "PLAY {} RTSP/1.0\r\nCSeq: {}\r\nAuthorization: {}\r\nUser-Agent: rtsp-invitor-1.0\r\nRange: npt=0-\r\nSession: {}\r\n\r\n",
            $url, $c_seq, $authorization, $session_id
        )
    };
}

#[macro_export]
macro_rules! teardown_request {
    ($url: expr, $c_seq: expr, $session_id: expr) => {
        format!(
            "TEARDOWN {} RTSP/1.0\r\nCSeq: {}\r\nUser-Agent: rtsp-invitor-1.0\r\nSession: {}\r\n\r\n",
            $url, $c_seq, $session_id
        )
    };
}

#[macro_export]
macro_rules! teardown_authenticate_request {
    ($url: expr, $c_seq: expr, $authorization: expr, $session_id: expr) => {
        format!(
            "TEARDOWN {} RTSP/1.0\r\nCSeq: {}\r\nAuthorization: {}\r\nUser-Agent: rtsp-invitor-1.0\r\nSession: {}\r\n\r\n",
            $url, $c_seq, $authorization, $session_id
        )
    };
}
