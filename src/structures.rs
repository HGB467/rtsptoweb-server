use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EncodeOptions{
    NONE,
    SINGLE,
    MULTI
}

#[derive(Deserialize,Clone)]
pub struct HLSOptions {
   pub max_files: u32,
    pub duration: u32
}

#[derive(Deserialize)]
pub struct StreamData {
    pub rtsp: String,
    pub stream_type: String,
    pub encode_options: Option<EncodeOptions>,
    pub hls_options: Option<HLSOptions>
}

#[derive(Serialize)]
pub struct ResponseData<T = ()> {
    pub status: bool,
    pub message: String,
    pub data: Option<T>
}

#[derive(Clone, Debug)]
pub struct RtspStream {
    pub status: bool,
    pub message: String,
    pub pipeline: Option<gstreamer::Pipeline>,
}

#[derive(Clone, Serialize, Debug)]
pub struct RtspResponse{
    pub status: bool,
    pub message: String
}