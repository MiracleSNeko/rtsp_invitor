#[macro_export]
macro_rules! option_request {
    ($url: expr, $c_seq: expr) => {
        format!(
            "OPTIONS {} RTSP/1.0\r\n
            CSeq: {}\r\n
            User-Agent: rtsp-invitor-1.0\r\n\r\n",
            $url, $c_seq
        )
    };
}

#[macro_export]
macro_rules! describe_request {
    ($url: expr, $c_seq: expr) => {
        format!(
            "DESCRIBE {} RTSP/1.0\r\n
            CSeq: {}\r\n
            User-Agent: rtsp-invitor-1.0\r\n\r\n",
            $url, $c_seq
        )
    };
}

#[macro_export]
macro_rules! describe_authenticate_request {
    ($url: expr, $c_seq: expr, $authorization: expr) => {
        format!(
            "DESCRIBE {} RTSP/1.0\r\n
            CSeq: {}\r\n
            User-Agent: rtsp-invitor-1.0\r\n
            Authorization: {}\r\n",
            $url, $c_seq, $authorization
        )
    };
}

#[macro_export]
macro_rules! setup_request {
    ($url: expr, $c_seq: expr, $port: expr) => {
        format!(
            "SETUP {} RTSP/1.0\r\n
            CSeq: {}\r\n
            User-Agent: rtsp-invitor-1.0\r\n
            Transport: rtp/udp;unicast;client_port={}-{}\r\n\r\n",
            $url,
            $c_seq,
            $port,
            $port + 1
        )
    };
}

#[macro_export]
macro_rules! play_request {
    ($url: expr, $c_seq: expr, $session_id: expr) => {
        format!(
            "PLAY {} RTSP/1.0\r\n
            CSeq: {}\r\n
            Range: npt=0-\r\n
            Session: {}\r\n
            User-Agent: rtsp-invitor-1.0\r\n\r\n",
            $url, $c_seq, $session_id
        )
    };
}

#[macro_export]
macro_rules! teardown_request {
    ($url: expr, $c_seq: expr, $session_id: expr) => {
        format!(
            "TEARDOWN {} RTSP/1.0\r\n
            CSeq: {}\r\n
            Session: {}\r\n
            User-Agent: rtsp-invitor-1.0\r\n\r\n",
            $url, $c_seq, $session_id
        )
    };
}
