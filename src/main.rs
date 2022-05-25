use crate::rtsp_machine::RtspMachine;
use std::io::{BufRead, BufReader, BufWriter};
use std::{collections::HashMap, env};

pub(crate) mod io_macros;
pub(crate) mod rtsp_camera;
pub(crate) mod rtsp_frame;
pub(crate) mod rtsp_machine;
pub(crate) mod rtsp_request;
pub(crate) mod rtsp_session;

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
        6,
        "Invalid input args! Usage: rtsp_invitor -url/-u <rtsp_url> -port/-p <rtsp_port:rtp_port> -camera/-c <camera>."
    );

    // Parse input
    let mut inputs = HashMap::new();
    args.as_slice()
        .windows(2)
        .step_by(2)
        .for_each(|input| {
            match input[0].as_str() {
                "-url" | "-u" => {
                    inputs.insert(String::from("Url"), input[1].clone());
                }
                "-port" | "-p" => {
                    inputs.insert(String::from("Port"), input[1].clone());
                }
                "-camera" | "-c" => {
                    inputs.insert(String::from("Camera"), input[1].clone());
                }
                _ => panic!("Invalid input args {}: {}! Usage: rtsp_invitor -url/-u <rtsp_url> -port/-p <rtsp_port:rtp_port> -camera/-c <camera>.", &input[0], &input[1]),
            }
        });

    // Create rtsp machine
    let mut rtsp_machine = RtspMachine::new(&inputs).await?;

    // Start rtsp machine
    let mut c_seq = 1;
    let mut repeat_request = 0;
    let mut repeat_response = 0;
    loop {
        // Send request
        let request = rtsp_machine.process_request(c_seq).await;

        // Check if request is send successfully
        match request {
            Ok(result) => {
                repeat_request = 0;
                // Break the loop and waiting for exit
                if !result {
                    break;
                }
            }
            Err(ref err) => {
                println!("Send request failed with error: {:?}, try again!", err);
                repeat_request += 1;
                if repeat_request >= 3 {
                    return Err(tokio::io::Error::new(
                        tokio::io::ErrorKind::Other,
                        "Send request failed after 3-times retry!",
                    ));
                }
                /* Send request failed, try again. */
                continue;
            }
        }

        // Clear buffer
        rtsp_machine.clear_buf();

        // Recv response
        let response = rtsp_machine.process_response(c_seq).await;

        // Check if response is recv successfully
        match response {
            Ok(c_seq_resp) => {
                repeat_response = 0;
                match c_seq_resp {
                    0 => { /* Recv response failed, send request again. */ }
                    _ => {
                        if c_seq_resp == c_seq + 1 {
                            // Recv response successfully
                            c_seq = c_seq_resp;
                        } else {
                            // CSeq mismatch, throw error
                            return Err(tokio::io::Error::new(
                                tokio::io::ErrorKind::Other,
                                "CSeq mismatch!",
                            ));
                        }
                    }
                }
            }
            Err(ref err) => {
                repeat_response += 1;
                println!("Recv response failed with error: {:?}, try again!", err);
                if repeat_response >= 3 {
                    return Err(tokio::io::Error::new(
                        tokio::io::ErrorKind::Other,
                        "Recv response failed after 3-times retry!",
                    ));
                }
                continue;
            }
        }

        // Clear buffer
        rtsp_machine.clear_buf();
    }

    println!("rtsp-invitor is done!");

    // Close rtsp machine when `exit` entered
    loop {
        println!("Enter `exit` to exit rtsp-invitor...");
        buf = getline!(cin_lock).unwrap_or_default();
        if buf.trim() == "exit" {
            rtsp_machine.shut_down(c_seq).await?;
            break;
        }
    }

    // Exit
    Ok(())
}
