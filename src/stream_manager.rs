

use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Query, State};
use axum::Json;
use gstreamer::prelude::{ElementExt, ElementExtManual};
use gstreamer_app::gst;
use tokio::sync::Mutex;
use crate::structures::{EncodeOptions, ResponseData, RtspResponse, RtspStream, StreamData};
use crate::streamer::{create_and_manage_playlists, create_hls_pipeline, create_webrtc_pipeline};

pub async fn add_stream(State(streams): State<Arc<Mutex<HashMap<String, RtspStream>>>>, Json(payload): Json<StreamData>) -> Json<ResponseData> {
    let formatted_encode_options = payload.encode_options.unwrap_or_else(|| EncodeOptions::NONE);
    let qualities = create_and_manage_playlists(payload.rtsp.clone(),formatted_encode_options.clone());

    let mut streams_lock = streams.lock().await;

    let formatted_rtsp = format!("{}-{}",payload.rtsp.clone(),payload.stream_type.clone());

    streams_lock.insert(formatted_rtsp.clone(), RtspStream{
        status: false,
        message: String::from("Starting"),
        pipeline: None
    });

    drop(streams_lock);

    tokio::spawn(async move {
        let create_pipeline = |stream_type: &str| -> Result<gst::Pipeline, String> {
            if stream_type == "HLS" {
                create_hls_pipeline(payload.rtsp.as_str(), qualities, formatted_encode_options, payload.hls_options)
            } else {
                create_webrtc_pipeline(payload.rtsp.as_str(),formatted_encode_options)
            }
        };

        match create_pipeline(payload.stream_type.as_str()) {
            Ok(pipeline) => {
                pipeline.set_state(gst::State::Playing).unwrap();

                let mut streams_lock = streams.lock().await;
                streams_lock.insert(
                    formatted_rtsp.clone(),
                    RtspStream {
                        status: true,
                        message: String::from("Started"),
                        pipeline: Some(pipeline.clone()),
                    },
                );
                drop(streams_lock);

                if let Some(bus) = pipeline.bus() {
                    bus.timed_pop_filtered(
                        gst::ClockTime::NONE,
                        &[gst::MessageType::Eos, gst::MessageType::Error],
                    );
                    let _ = pipeline.set_state(gst::State::Null);
                    let mut streams_lock = streams.lock().await;

                    if streams_lock.contains_key(formatted_rtsp.as_str()) {
                        streams_lock.insert(formatted_rtsp.clone(),RtspStream{
                            status: false,
                            message: String::from("Pipeline Ended"),
                            pipeline: None,
                        });
                    };

                    drop(streams_lock);
                } else {
                    pipeline.set_state(gst::State::Null).unwrap();
                    let mut streams_lock = streams.lock().await;
                    streams_lock.insert(
                        formatted_rtsp.clone(),
                        RtspStream {
                            status: false,
                            message: String::from("Bus not initialized"),
                            pipeline: None,
                        },
                    );
                    drop(streams_lock);
                }
            }
            Err(e) => {
                let mut streams_lock = streams.lock().await;
                streams_lock.insert(
                    formatted_rtsp.clone(),
                    RtspStream {
                        status: false,
                        message: e,
                        pipeline: None,
                    },
                );
                drop(streams_lock);
            }
        }

    });

    let response = ResponseData {
        status: true,
        message: String::from("Initiated"),
        data: None
    };

    Json(response)
}

pub async fn delete_stream(
    Query(params): Query<StreamData>,
    State(streams): State<Arc<Mutex<HashMap<String, RtspStream>>>>,
) -> Json<ResponseData> {
    let mut streams_lock = streams.lock().await;

    let formatted_rtsp = format!("{}-{}",params.rtsp.clone(),params.stream_type.clone());

    if let Some(stream) = streams_lock.get(formatted_rtsp.as_str()) {
        if let Some(pipeline) = &stream.pipeline {
            pipeline.send_event(gst::event::Eos::new());
        }

        streams_lock.remove(formatted_rtsp.as_str());

        drop(streams_lock);

        Json(ResponseData {
            status: true,
            message: format!("Stream '{}' deleted successfully", &params.rtsp),
            data: None
        })
    }
    else{
        Json(ResponseData {
            status: true,
            message: String::from("Stream not found"),
            data: None
        })
    }
}
pub async fn get_streams(
    State(streams): State<Arc<Mutex<HashMap<String, RtspStream>>>>,
) -> Json<ResponseData<HashMap<String, RtspResponse>>> {
    let data = streams.lock().await;

    let mut new_data : HashMap<String,RtspResponse> = HashMap::new();

    for (key, value) in data.iter() {
        new_data.insert(key.clone(),RtspResponse{
            status: value.status,
            message: value.message.clone()
        });
    }

    Json(ResponseData{
        status: true,
        message: String::from("Fetch Success"),
        data: Some(new_data.clone())
    })
}