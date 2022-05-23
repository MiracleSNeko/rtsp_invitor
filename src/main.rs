use crate::{
    rtsp_camera::{AxisCamera, EstablishRtspConnection, RtspCamera},
    rtsp_machine::{RtspMachine, RtspState},
};
use sscanf::scanf;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{collections::HashMap, env};
use tokio::{
    io::{Interest, Result},
    net::TcpStream,
};

pub(crate) mod io_macros;
pub(crate) mod rtsp_camera;
pub(crate) mod rtsp_frame;
pub(crate) mod rtsp_machine;
pub(crate) mod rtsp_request;
pub(crate) mod rtsp_response;
pub(crate) mod rtsp_session;

#[allow(unreachable_code)]
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
        8,
        "Invalid input args! Usage: rtsp_invitor -url/-u <rtsp_url> -output/-o <ip.dst:port> -camera/-c <camera>."
    );

    // Parse input
    let mut inputs = HashMap::new();
    args.as_slice()
        .windows(2)
        .for_each(|input| match input[0].as_str() {
            "-url" | "-u" => {
                inputs.insert(String::from("Url"), input[1].clone());
            }
            "-out" | "-o" => {
                inputs.insert(String::from("Output"), input[1].clone());
            }
            "-camera" | "-c" => {
                inputs.insert(String::from("Camera"), input[1].clone());
            }
            _ => panic!("Invalid input args! Usage: rtsp_invitor -url/-u <rtsp_url> -output/-o <ip.dst:port> -camera/-c <camera>."),
        });

    // Create rtsp machine
    let mut rtsp_machine = RtspMachine::new(&inputs).await?;

    // Start rtsp machine
    let mut c_seq = 1;
    loop {
        // Send request
        let request = rtsp_machine.process_request(c_seq).await;

        // Check if request is send successfully
        match request {
            Ok(_) => { /* Do nothing */ },
            // BUG: cause `main` function is not allowed to be `async`
            //      find a better way to exit the loop
            // Err(ref err) if err.kind() == tokio::io::ErrorKind::Other && err.to_string() == String::from("Cannot send TEARDOWN request in `process_request`") {
            //     // All request has been sent except TEARDOWN
            //     break;
            // },
            Err(ref err) => {
                println!("Send request failed with error {}, try again!", err);
                /* Send request failed, try again. */
                continue;
            }
        }

        // Recv response
        let response = rtsp_machine.process_response(c_seq).await;

        // Check if response is recv successfully
        match response {
            Ok(c_seq_resp) => match c_seq_resp {
                0 => { /* Recv response failed, send request again. */ },
                 _ => {
                    if c_seq_resp == c_seq + 1 {
                        // Recv response successfully
                        c_seq = c_seq_resp;
                    } else {
                        // CSeq mismatch, throw error
                        return Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, "CSeq mismatch!"));
                    }
                }
            },
            Err(ref err) => todo!()
        }
    }

    // 
    println!("rtsp-invitor is done!");

    // // Close rtsp machine when `exit` entered
    // loop {
    //     println!("Enter `exit` to exit rtsp-invitor...");
    //     let mut input = getline!(cin_lock).unwrap_or_default();
    //     if input.trim() == "exit" {
    //         rtsp_machine.shut_down(c_seq).await?;
    //         break;
    //     }
    // }

    // Exit
    Ok(())
}
