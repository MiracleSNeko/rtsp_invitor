// use crate::rtsp_url_parser::RtspConnection;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RtspState {
    Starting,
    OptionsRecv,
    DescribeSend,
    DescribeRecv,
    UnauthorizedRecv,
    AuthorizationSend,
    SetupSend,
    SetupRecv,
    PlaySend,
    PlayRecv,
    TeardownSend,
    Exiting,
}

// #[derive(Debug)]
// pub(crate) struct RtspMachine {
//     pub(crate) rtsp_connection: RtspConnection,
//     pub(crate) rtsp_state: RtspState,
// }
