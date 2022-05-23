use std::io::Cursor;

use crate::rtsp_frame::RtspFrame;
use bytes::{BytesMut, Buf};
use tokio::io::{AsyncReadExt, Error, ErrorKind, Result, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub(crate) struct RtspSession {
    pub(crate) stream: TcpStream,
    pub(crate) buf: BytesMut,
}

impl RtspSession {
    /// `RtspSession` is a struct that contains a `TcpStream` and a `BytesMut` buffer. The `new`
    /// function creates a new `RtspSession` by taking a `TcpStream` and creating a `BytesMut` buffer
    /// with a capacity of 1500 bytes
    ///
    /// Arguments:
    ///
    /// * `stream`: TcpStream
    ///
    /// Returns:
    ///
    /// A new RtspSession struct.
    pub(crate) fn new(stream: TcpStream) -> RtspSession {
        RtspSession {
            stream,
            buf: BytesMut::with_capacity(1500), // length of rtsp frame <= MTU
        }
    }

    // write rtsp request to TcpStream
    pub(crate) async fn write_frame(&mut self, frame: &RtspFrame) -> Result<()> {
        let mut buf = BytesMut::with_capacity(1500);
        let len = frame.assemble_request(&mut buf)?;
        self.stream.write_all(&buf[..len]).await?;
        Ok(())
    }

    // read rtsp response from TcpStream
    pub(crate) async fn read_frame(&mut self) -> Result<Option<RtspFrame>> {
        loop {
            if let Some(frame) = self.parse_frame().await? {
                return Ok(Some(frame));
            }
            if 0 == self.stream.read_buf(&mut self.buf).await? {
                return Ok(None);
            } else {
                return Err(Error::new(ErrorKind::Other, "Failed to read frame"));
            }
        }
    }

    async fn parse_frame(&mut self) -> Result<Option<RtspFrame>> {
        // Check if the first line is a valid RTSP response
        match RtspFrame::check_response(&self.buf) {
            Ok(_) => {
                let mut cursor = Cursor::new(&self.buf[..]);
                cursor.set_position(0);
                let (frame, len) = RtspFrame::parse_response(&mut cursor).await?;
                self.buf.advance(len);
                Ok(Some(frame))
            }
            Err(err) => Err(err)
        }
    }
}
