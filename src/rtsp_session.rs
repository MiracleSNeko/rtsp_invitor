use crate::rtsp_frame::RtspFrame;
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind, Result};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

#[derive(Debug)]
pub(crate) struct RtspSession {
    pub(crate) reader: OwnedReadHalf,
    pub(crate) writer: OwnedWriteHalf,
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
        let (reader, writer) = stream.into_split();
        RtspSession {
            reader,
            writer,
            buf: BytesMut::with_capacity(1500), // length of rtsp frame <= MTU
        }
    }

    /// This function takes a frame, assembles it into a buffer, and then writes it to the socket
    /// 
    /// Arguments:
    /// 
    /// * `frame`: &RtspFrame
    /// 
    /// Returns:
    /// 
    /// The number of bytes written.
    pub(crate) async fn write_frame(&mut self, frame: &RtspFrame) -> Result<usize> {
        let len = frame.assemble_request(&mut self.buf)?;
        println!(
            "Sending request:\n{}",
            String::from_utf8_lossy(&self.buf[..len])
        );
        self.writer.write_all(&self.buf[..len]).await?;
        self.writer.flush().await?;
        Ok(len)
    }

    /// Read from the TCP stream until we get a non-empty buffer, then parse the buffer into a frame
    /// 
    /// Returns:
    /// 
    /// a Result<Option<RtspFrame>>.
    pub(crate) async fn read_frame(&mut self) -> Result<Option<RtspFrame>> {
        loop {
            match self.reader.read_buf(&mut self.buf).await {
                Ok(_len) => {
                    // println!("Read {} bytes from TcpStream.", len);
                    break;
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    /* do nothing */
                    println!(
                        "Read from TcpStream throw Error::WouldBlock, waiting for next read..."
                    );
                    continue;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        println!("Received response:\n{}", String::from_utf8_lossy(&self.buf));
        if self.buf.is_empty() {
            return Ok(None);
        } else {
            self.parse_frame().await
        }
    }

    /// If the first line of the buffer is a valid RTSP response, parse it and return it
    /// 
    /// Returns:
    /// 
    /// A tuple of the frame and the length of the frame.
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
            Err(err) => Err(err),
        }
    }
}
