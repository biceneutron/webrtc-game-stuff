use clap::{Arg, Command};

use bytes::Bytes;
use log::warn;
use reqwest::{Client as HttpClient, Response as HttpResponse};
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc, time::sleep};
use webrtc::{
    api::{setting_engine::SettingEngine, APIBuilder},
    data::data_channel::DataChannel,
    data_channel::{
        data_channel_init::RTCDataChannelInit, data_channel_message::DataChannelMessage,
        RTCDataChannel,
    },
    dtls_transport::dtls_role::DTLSRole,
    error::Error as RTCError,
    ice::mdns::MulticastDnsMode,
    ice_transport::{
        ice_candidate::{RTCIceCandidate, RTCIceCandidateInit},
        ice_server::RTCIceServer,
    },
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
};

use serde::{Deserialize, Serialize};
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct SdpResponse {
    sdp: RTCSessionDescription,
    candidates: Vec<RTCIceCandidate>,
}

#[tokio::main]
async fn main() -> Result<(), RTCError> {
    let args: Vec<String> = std::env::args().collect();

    let exec_type = &args[1];
    println!("exec_type {}", exec_type);

    let matches: clap::ArgMatches = Command::new("app")
        .arg(
            Arg::new("server")
                .short('s')
                .long("server")
                .takes_value(true)
                .required(false)
                .help("server address"),
        )
        .get_matches();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::debug!("Client running!");
    //
    // get args
    //
    let server_url: String = matches
        .value_of("server")
        .unwrap()
        .parse()
        .expect("could not parse server data address/port");

    //
    // connect
    //

    // let (to_server_sender, to_server_receiver) = mpsc::unbounded_channel();
    // let (to_client_sender, to_client_receiver) = mpsc::unbounded_channel();

    // let mut m = MediaEngine::default();

    // create a SettingEngine and enable Detach
    let mut setting_engine = SettingEngine::default();
    setting_engine.set_srtp_protection_profiles(vec![]);
    // setting_engine.detach_data_channels();
    setting_engine.set_ice_multicast_dns_mode(MulticastDnsMode::Disabled);
    setting_engine
        .set_answering_dtls_role(DTLSRole::Client)
        .expect("error in set_answering_dtls_role!");

    // Register default codecs
    // m.register_default_codecs()?;

    // let mut registry = Registry::new();

    // Use the default set of Interceptors
    // registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        // .with_media_engine(m)
        // .with_interceptor_registry(registry)
        .with_setting_engine(setting_engine)
        .build();

    // Prepare the configuration
    // stun:stun.l.google.com:19302?transport=udp
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            // urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            // urls: vec!["stun:stun.l.google.com:19302?transport=udp".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // create a new RTCPeerConnection
    // let peer_connection = RTCPeerConnection::new().await;

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    let label = "data";
    let protocol = "";

    // create a datachannel with label 'data'
    let data_channel = peer_connection
        .create_data_channel(
            label,
            Some(RTCDataChannelInit {
                ordered: Some(false),
                max_retransmits: Some(1),
                // max_packet_life_time: Some(500),
                ..Default::default()
            }),
        )
        .await
        .expect("cannot create data channel");

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        log::debug!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
            // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
            // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
            log::debug!("Peer Connection has gone to failed exiting");
            let _ = done_tx.try_send(());
        }

        Box::pin(async {})
    }));

    // datachannel on_error callback
    data_channel.on_error(Box::new(move |error| {
        log::error!("data channel error: {:?}", error);
        Box::pin(async {})
    }));
    // .await;

    // datachannel on_open callback
    let data_channel_ref_1 = Arc::clone(&data_channel);
    data_channel.on_open(Box::new(move || {
        // log::debug!("data_channel.on_open");
        // let data_channel_ref_2 = Arc::clone(&data_channel_ref);
        // Box::pin(async move {
        //     let detached_data_channel = data_channel_ref_2
        //         .detach()
        //         .await
        //         .expect("data channel detach got error");

            // Handle reading from the data channel
            // let detached_data_channel_1 = Arc::clone(&detached_data_channel);
            // let detached_data_channel_2 = Arc::clone(&detached_data_channel);
            // tokio::spawn(async move {
            //     let _loop_result = read_loop(detached_data_channel_1, to_client_sender).await;
            //     // do nothing with result, just close thread
            // });

        //     // Handle writing to the data channel
        //     tokio::spawn(async move {
        //         let _loop_result = write_loop(detached_data_channel_2, to_server_receiver).await;
        //         // do nothing with result, just close thread
        //     });
        // })
        println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 2 seconds", data_channel_ref_1.label(), data_channel_ref_1.id());

        let data_channel_ref_2 = Arc::clone(&data_channel_ref_1);
        Box::pin(async move {
            let mut result = Result::<usize, RTCError>::Ok(0);
            let mut packet_seq = 0;
            while result.is_ok() {
                let timeout = tokio::time::sleep(Duration::from_secs(2));
                tokio::pin!(timeout);

                tokio::select! {
                    _ = timeout.as_mut() =>{
                        let message = format!("CLIENT_PACKET_{}", packet_seq);
                        println!("Sending '{message}'");
                        // result = data_channel_ref_2.send_text(message).await.map_err(Into::into);
                        result = data_channel_ref_2.send(&Bytes::from(message)).await.map_err(Into::into);
                        packet_seq += 1;
                    }
                };
            }
        })
    }));
    // .await;

    // Register text message handling
    let d_label = data_channel.label().to_owned();
    data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
        let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
        println!("Received from server, '{d_label}': {msg_str}");
        Box::pin(async {})
    }));

    // peer_connection's on_ice_candidate callback
    peer_connection.on_ice_candidate(Box::new(move |candidate_opt| {
        if let Some(candidate) = &candidate_opt {
            log::debug!("received ice candidate from: {}", candidate.address);
        } else {
            log::debug!("all local candidates received");
        }

        Box::pin(async {})
    }));
    // .await;

    // create an offer to send to the server
    let offer = peer_connection
        .create_offer(None)
        .await
        .expect("cannot create offer");

    // log::debug!("offer {}", offer.sdp);

    // sets the LocalDescription, and starts our UDP listeners
    peer_connection
        .set_local_description(offer)
        .await
        .expect("cannot set local description");

    // send a request to server to initiate connection (signaling, essentially)
    let http_client = HttpClient::new();

    // let sdp: String = peer_connection.local_description().await.unwrap().sdp;
    let sdp = peer_connection.local_description().await.unwrap();

    // // log::debug!("sdp {}", sdp);

    // let sdp_len = sdp.len();

    let payload = match serde_json::to_string(&sdp) {
        Ok(p) => p,
        Err(err) => panic!("{}", err),
    };

    // wait to receive a response from server
    let response: HttpResponse = loop {
        // let request = http_client
        //     .post(server_url.clone())
        //     .header("Content-Length", sdp_len)
        //     .body(sdp.clone());

        let request = http_client
            .post(server_url.clone())
            .header("content-type", "application/json; charset=utf-8")
            .body(payload.clone());

        match request.send().await {
            Ok(resp) => {
                break resp;
            }
            Err(err) => {
                warn!("Could not send request, original error: {:?}", err);
                sleep(Duration::from_secs(1)).await;
            }
        };
    };
    let response_string = response.text().await.unwrap();

    // parse session from server response
    // let session_response: JsSessionResponse = get_session_response(response_string.as_str());

    // // log::debug!("answer {}", session_response.answer.sdp);
    // // log::debug!("candidate {}", session_response.candidate.candidate);

    // apply the server's response as the remote description
    // let session_description = RTCSessionDescription::answer(session_response.answer.sdp).unwrap();

    let sdp_response = match serde_json::from_str::<SdpResponse>(&response_string) {
        Ok(s) => s,
        Err(err) => panic!("{}", err),
    };

    // print!("session_description.sdp {}", session_description.sdp);

    peer_connection
        // .set_remote_description(session_description)
        .set_remote_description(sdp_response.sdp)
        .await
        .expect("cannot set remote description");

    // println!(
    //     "session_response.candidate.candidate: {}",
    //     session_response.candidate.candidate
    // );
    // println!(
    //     "session_response.candidate.sdp_mid: {}",
    //     session_response.candidate.sdp_mid
    // );
    // println!(
    //     "session_response.candidate.sdp_m_line_index: {}",
    //     session_response.candidate.sdp_m_line_index
    // );

    // add ice candidate to connection
    if let Err(error) = peer_connection
        // .add_ice_candidate(session_response.candidate.candidate)
        .add_ice_candidate(RTCIceCandidateInit {
            // candidate: session_response.candidate.candidate,
            // sdp_mid: Some(session_response.candidate.sdp_mid),
            // sdp_mline_index: Some(session_response.candidate.sdp_m_line_index),
            candidate: sdp_response.candidates[0].to_json().unwrap().candidate,
            ..Default::default()
        })
        .await
    {
        panic!("Error during add_ice_candidate: {:?}", error);
    }

    // println!("Press ctrl-c to stop");
    // tokio::select! {
    //     _ = done_rx.recv() => {
    //         println!("received done signal!");
    //     }
    //     _ = tokio::signal::ctrl_c() => {
    //         println!();
    //     }
    // };
    // peer_connection.close().await?;
    // Ok(())

    loop {}
}

const MESSAGE_SIZE: usize = 1500;

// async fn read_loop(
//     data_channel: Arc<DataChannel>,
//     to_client_sender: mpsc::UnboundedSender<Box<[u8]>>,
// ) -> Result<(), RTCError> {
//     let mut buffer = vec![0u8; MESSAGE_SIZE];
//     loop {
//         let message_length = match data_channel.read(&mut buffer).await {
//             Ok(length) => length,
//             Err(err) => {
//                 println!("Datachannel closed; Exit the read_loop: {}", err);
//                 return Ok(());
//             }
//         };

//         match to_client_sender.send(buffer[..message_length].into()) {
//             Ok(_) => {}
//             Err(e) => {
//                 return Err(RTCError::new(e.to_string()));
//             }
//         }
//     }
// }

use std::str;
async fn read_loop(
    data_channel: Arc<DataChannel>,
    to_client_sender: mpsc::UnboundedSender<Box<[u8]>>,
) -> Result<(), RTCError> {
    let mut buffer = vec![0u8; MESSAGE_SIZE];
    loop {
        println!("#### read_loop ####");
        let message_length = match data_channel.read(&mut buffer).await {
            Ok(length) => length,
            Err(err) => {
                println!("Datachannel closed; Exit the read_loop: {}", err);
                return Ok(());
            }
        };

        if let Ok(message) = String::from_utf8(buffer[..message_length].to_vec()) {
            println!(
                "Received from server, {} bytes: {}",
                message_length, message
            );
        } else {
            println!(
                "Received from server, {} bytes, but cannot parse it",
                message_length
            );
        }
    }
    println!("#### end read_loop ####");
}

// write_loop shows how to write to the datachannel directly
// async fn write_loop(
//     data_channel: Arc<DataChannel>,
//     mut to_server_receiver: mpsc::UnboundedReceiver<Box<[u8]>>,
// ) -> Result<(), RTCError> {
//     loop {
//         if let Some(write_message) = to_server_receiver.recv().await {
//             match data_channel.write(&Bytes::from(write_message)).await {
//                 Ok(_) => {}
//                 Err(e) => {
//                     return Err(RTCError::new(e.to_string()));
//                 }
//             }
//         } else {
//             return Ok(());
//         }
//     }
// }

async fn write_loop(
    data_channel: Arc<DataChannel>,
    mut to_server_receiver: mpsc::UnboundedReceiver<Box<[u8]>>,
) -> Result<(), RTCError> {
    let mut packet_seq = 0;
    loop {
        println!("#### write_loop ####");
        if packet_seq < 10 {
            match data_channel.write(&Bytes::from("CLIENT_PACKET")).await {
                Ok(_) => {
                    packet_seq += 1;
                }
                Err(e) => {
                    return Err(RTCError::new(e.to_string()));
                }
            }
        } else {
            return Ok(());
        }
    }
    println!("#### write_loop ####");
}

struct SessionAnswer {
    pub sdp: String,
    pub type_str: String,
}

struct SessionCandidate {
    pub candidate: String,
    pub sdp_m_line_index: u16,
    pub sdp_mid: String,
}

struct JsSessionResponse {
    pub(crate) answer: SessionAnswer,
    pub(crate) candidate: SessionCandidate,
}

use tinyjson::JsonValue;
fn get_session_response(input: &str) -> JsSessionResponse {
    let json_obj: JsonValue = input.parse().unwrap();

    let sdp_opt: Option<&String> = json_obj["answer"]["sdp"].get();
    let sdp: String = sdp_opt.unwrap().clone();

    let type_str_opt: Option<&String> = json_obj["answer"]["type"].get();
    let type_str: String = type_str_opt.unwrap().clone();

    let candidate_opt: Option<&String> = json_obj["candidate"]["candidate"].get();
    let candidate: String = candidate_opt.unwrap().clone();

    let sdp_m_line_index_opt: Option<&f64> = json_obj["candidate"]["sdpMLineIndex"].get();
    let sdp_m_line_index: u16 = *(sdp_m_line_index_opt.unwrap()) as u16;

    let sdp_mid_opt: Option<&String> = json_obj["candidate"]["sdpMid"].get();
    let sdp_mid: String = sdp_mid_opt.unwrap().clone();

    JsSessionResponse {
        answer: SessionAnswer { sdp, type_str },
        candidate: SessionCandidate {
            candidate,
            sdp_m_line_index,
            sdp_mid,
        },
    }
}
