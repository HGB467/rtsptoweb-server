


    use std::fs;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use gstreamer as gst;
    use gstreamer::{Element, Pipeline};
    use gstreamer::prelude::*;
    use crate::structures::{EncodeOptions, HLSOptions};

    // default values
    const MAX_FILES: u32 = 17280;
    const DURATION: u32 = 2;


    pub struct Quality{
        width: i32,
        height: i32,
        bitrate: u32
    }

    fn create_dynamic_pipeline(parser: Element,encode_options: EncodeOptions, sink: Element, pipeline: Pipeline) -> Result<(),String>{
        match encode_options {
            EncodeOptions::NONE => {
                if let Some(sink_pad) = sink.request_pad_simple("video_%u") {
                    parser.connect("pad-added", false, move |values| {
                        let pad = match values[1].get::<gst::Pad>() {
                            Ok(pad) => pad,
                            Err(_) => return None,
                        };

                        if !sink_pad.is_linked() {
                            let _ = pad.link(&sink_pad);
                        }

                        None
                    });
                }
                else{
                    return Err(String::from("Could not get webrtc sink pads"));
                }

                Ok(())
            }
            EncodeOptions::SINGLE => {
                let dec = match gst::ElementFactory::make("decodebin").build() {
                    Ok(element) => element,
                    Err(_) => return Err("Failed to create decodebin".to_string()),
                };

                let conv = match gst::ElementFactory::make("videoconvert").build() {
                    Ok(element) => element,
                    Err(_) => return Err("Failed to create videoconvert".to_string()),
                };
                let enc = match gst::ElementFactory::make("x264enc").build() {
                    Ok(element) => element,
                    Err(_) => return Err("Failed to create x264enc".to_string()),
                };

                let queue = match gst::ElementFactory::make("queue").build() {
                    Ok(element) => element,
                    Err(_) => return Err("Failed to create queue".to_string()),
                };

                if let Err(e) = pipeline.add_many(&[&dec,&conv,&queue, &enc]) {
                     return Err(format!("Failed to add elements to pipeline: {:?}",e));
                }


                if let Err(_) = Element::link_many(&[&conv, &enc, &queue, &sink]) {
                    return Err("Failed to link elements".to_string());
                }

                let d = dec.clone();

                parser.connect("pad-added", false, move |values| {
                    let pad = match values[1].get::<gst::Pad>() {
                        Ok(pad) => pad,
                        Err(_) => return None,
                    };

                    if let Some(sink_pad) = d.static_pad("sink") {
                        if !sink_pad.is_linked() {
                            let _ = pad.link(&sink_pad);
                        }
                    }


                    None
                });

                dec.connect("pad-added", false, move |values| {
                    let pad = match values[1].get::<gst::Pad>() {
                        Ok(pad) => pad,
                        Err(_) => return None,
                    };

                    if let Some(sink_pad) = conv.static_pad("sink") {
                        if !sink_pad.is_linked() {
                            let _ = pad.link(&sink_pad);
                        }
                    }


                    None
                });

                Ok(())

            }
            EncodeOptions::MULTI => {
                let dec = match gst::ElementFactory::make("decodebin").build() {
                    Ok(element) => element,
                    Err(_) => return Err("Failed to create decodebin".to_string()),
                };

                let conv = match gst::ElementFactory::make("videoconvert").build() {
                    Ok(element) => element,
                    Err(_) => return Err("Failed to create videoconvert".to_string()),
                };

                if let Err(e) = pipeline.add_many(&[&dec,&conv]) {
                    return Err(format!("Failed to add elements to pipeline: {:?}", e));
                }


                if let Err(_) = Element::link_many(&[&conv, &sink]) {
                    return Err("Failed to link elements".to_string());
                }

                let d = dec.clone();

                parser.connect("pad-added", false, move |values| {
                    let pad = match values[1].get::<gst::Pad>() {
                        Ok(pad) => pad,
                        Err(_) => return None,
                    };

                    if let Some(sink_pad) = d.static_pad("sink") {
                        if !sink_pad.is_linked() {
                            let _ = pad.link(&sink_pad);
                        }
                    }


                    None
                });

                dec.connect("pad-added", false, move |values| {
                    let pad = match values[1].get::<gst::Pad>() {
                        Ok(pad) => pad,
                        Err(_) => return None,
                    };

                    if let Some(sink_pad) = conv.static_pad("sink") {
                        if !sink_pad.is_linked() {
                            let _ = pad.link(&sink_pad);
                        }
                    }


                    None
                });


                Ok(())
            }
        }


    }

    pub fn create_webrtc_pipeline(rtsp: &str, encode_options: EncodeOptions) -> Result<Pipeline, String> {
        let pipeline = Pipeline::new();

        let src = match gst::ElementFactory::make("rtspsrc").build() {
            Ok(element) => element,
            Err(_) => return Err("Failed to create rtspsrc".to_string()),
        };
        src.set_property("location", rtsp);

        let parse = match gst::ElementFactory::make("parsebin").build() {
            Ok(element) => element,
            Err(_) => return Err("Failed to create parsebin".to_string()),
        };
        let sink = match gst::ElementFactory::make("webrtcsink").build() {
            Ok(element) => element,
            Err(_) => return Err("Failed to create webrtcsink".to_string()),
        };
        sink.set_property("name", rtsp);

        let mut meta = gst::Structure::new_empty("meta");
        meta.set("rtsp",rtsp);

        sink.set_property("meta", meta);

        if let Err(e) = pipeline.add_many(&[&src,&parse, &sink]) {
            return Err(format!("Failed to add elements to pipeline: {:?}", e));
        }

        let p = parse.clone();

        src.connect("pad-added", false, move |values| {
            let pad = match values[1].get::<gst::Pad>() {
                Ok(pad) => pad,
                Err(_) => return None,
            };

            if let Some(sink_pad) = p.static_pad("sink") {
                if !sink_pad.is_linked() {
                    let _ = pad.link(&sink_pad);
                }
            }


            None
        });


        if let Err(E) = create_dynamic_pipeline(parse, encode_options, sink,pipeline.clone()){
            return Err(E)
        }

        Ok(pipeline)
    }



    fn create_quality_elements(quality: Quality, pipeline: Pipeline, tee: &Element, rtsp: String, hls_options: HLSOptions){
        let queue = gst::ElementFactory::make("queue").build().unwrap();
        let scale = gst::ElementFactory::make("videoscale").build().unwrap();
        let caps = gst::Caps::builder("video/x-raw")
            .field("width", quality.width)
            .field("height", quality.height)
            .build();
        let filter = gst::ElementFactory::make("capsfilter").build().unwrap();
        filter.set_property("caps", &caps);

        let convert = gst::ElementFactory::make("videoconvert").build().unwrap();
        let encode = gst::ElementFactory::make("x264enc").build().unwrap();
        encode.set_property("bitrate",quality.bitrate);

        let queue1 = gst::ElementFactory::make("queue").build().unwrap();

        let parse = gst::ElementFactory::make("h264parse").build().unwrap();

        let rtsp_dir = format!("{}/{}", "./hls", rtsp.replace(&['/', ':', '?', '&'][..], "_"));

        let hlssink = gst::ElementFactory::make("hlssink2").build().unwrap();
        hlssink.set_property("location", format!("{}/{}p/segment%05d.ts",rtsp_dir,quality.height));
        hlssink.set_property("playlist-location", format!("{}/{}p/playlist.m3u8",rtsp_dir,quality.height));
        hlssink.set_property("target-duration", hls_options.duration);
        hlssink.set_property("max-files", hls_options.max_files);
        hlssink.set_property("playlist-length", hls_options.max_files);

        pipeline.add_many(&[
            &queue,
            &scale,
            &filter,
            &convert,
            &queue1,
            &encode,
            &parse,
            &hlssink,
        ]).unwrap();

        Element::link_many(&[
            tee,
            &queue,
            &convert,
            &scale,
            &filter,
            &encode,
            &queue1,
            &parse,
            &hlssink,
        ]).unwrap();
    }


    pub fn create_hls_pipeline(rtsp: &str, qualities: Vec<Quality>, encode_options: EncodeOptions, hls_options: Option<HLSOptions> ) -> Result<Pipeline, String> {
        let pipeline = Pipeline::new();

        let formatted_hls_options = hls_options.unwrap_or_else(|| HLSOptions{
            max_files: MAX_FILES,
            duration: DURATION
        });

        let rtspsrc = match gst::ElementFactory::make("rtspsrc").build() {
            Ok(element) => element,
            Err(_) => return Err("Failed to create rtspsrc".to_string()),
        };
        rtspsrc
            .set_property("location", rtsp);

        let parse = match gst::ElementFactory::make("parsebin").build() {
            Ok(element) => element,
            Err(_) => return Err("Failed to create parsebin".to_string()),
        };


        if let Err(e) = pipeline.add_many(&[&rtspsrc, &parse]) {
            return Err(format!("Failed to add elements to pipeline: {:?}", e));
        }

        let sink ;

        if let EncodeOptions::NONE = encode_options {
            let rtsp_dir = format!("{}/{}", "./hls", rtsp.replace(&['/', ':', '?', '&'][..], "_"));

            let hlssink = gst::ElementFactory::make("hlssink2").build().unwrap();
            hlssink.set_property("location", format!("{}/{}p/segment%05d.ts",rtsp_dir,1080));
            hlssink.set_property("playlist-location", format!("{}/{}p/playlist.m3u8",rtsp_dir,1080));
            hlssink.set_property("target-duration", formatted_hls_options.duration);
            hlssink.set_property("max-files", formatted_hls_options.max_files);
            hlssink.set_property("playlist-length", formatted_hls_options.max_files);

            if let Err(e) = pipeline.add_many(&[&hlssink]){
                return Err(format!("Failed to add elements to pipeline: {:?}", e));
            }

            sink = hlssink;
        }
        else{
            let decode = match gst::ElementFactory::make("decodebin").build() {
                Ok(element) => element,
                Err(_) => return Err("Failed to create decodebin".to_string()),
            };


            let tee = match gst::ElementFactory::make("tee").build() {
                Ok(element) => element,
                Err(_) => return Err("Failed to create tee".to_string()),
            };


            if let Err(e) = pipeline.add_many(&[&decode, &tee]) {
                return Err(format!("Failed to add elements to pipeline: {:?}", e));
            }


            for quality in qualities {
                create_quality_elements(quality, pipeline.clone(), &tee, rtsp.to_string(), formatted_hls_options.clone());
            }

            decode.connect("pad-added", false, move |values| {
                let pad = match values[1].get::<gst::Pad>() {
                    Ok(pad) => pad,
                    Err(_) => return None,
                };

                if let Some(sink_pad) = tee.static_pad("sink") {
                    if !sink_pad.is_linked() {
                        let _ = pad.link(&sink_pad);
                    }
                }


                None
            });

            sink = decode;
        }


        let p = parse.clone();

        rtspsrc.connect("pad-added", false, move |values| {
            let pad = match values[1].get::<gst::Pad>() {
                Ok(pad) => pad,
                Err(_) => return None,
            };


            if let Some(sink_pad) = p.static_pad("sink") {
                if !sink_pad.is_linked() {
                    let _ = pad.link(&sink_pad);
                }
            }


            None
        });

        let d = sink.clone();
        let pipe = pipeline.clone();

        parse.connect("pad-added", false, move |values| {
            let pad = match values[1].get::<gst::Pad>() {
                Ok(pad) => pad,
                Err(_) => return None,
            };

            let caps = pad.query_caps(None);
            let structure = caps.structure(0).unwrap();
            let name = structure.name();
            let encoding = name.as_str();

            let parser;

            match encoding {
                "video/x-h264" => {
                    parser = match gst::ElementFactory::make("h264parse").build() {
                        Ok(element) => element,
                        Err(_) => return None,
                    };
                }
                "video/x-h265" => {
                    parser = match gst::ElementFactory::make("h265parse").build() {
                        Ok(element) => element,
                        Err(_) => return None,
                    };
                }
                _ => {
                    return None;
                }
            }


            if let Err(e) = pipe.add_many(&[&parser]){
                return None;
            }

            Element::link_many(&[
                &parser,
                &d
            ]).unwrap();

            if let Some(sink_pad) = parser.static_pad("sink") {
                if !sink_pad.is_linked() {
                        let _ = pad.link(&sink_pad);
                }
            }

            let _ = parser.set_state(gst::State::Playing);

            None
        });



        Ok(pipeline)
    }


    pub fn create_and_manage_playlists(rtsp: String, encode_options: EncodeOptions)-> Vec<Quality>{
        let mut qualities = vec![];

        if let EncodeOptions::MULTI = encode_options {
            qualities.extend([
                Quality { width: 1920, height: 1080, bitrate: 4000 },
                Quality { width: 1280, height: 720, bitrate: 2500 },
                Quality { width: 640, height: 480, bitrate: 1000 },
            ]);
        }
        else{
            qualities.push(Quality { width: 1920, height: 1080, bitrate: 4000 });
        }

        let hls_dir = "./hls";

        let rtsp_dir = format!("{}/{}", hls_dir, rtsp.replace(&['/', ':', '?', '&'][..], "_"));
        if Path::new(&rtsp_dir).exists() {
            fs::remove_dir_all(&rtsp_dir).unwrap();
        }

        for quality in &qualities {
            let quality_dir = format!("{}/{}p", rtsp_dir, quality.height);
            if Path::new(&quality_dir).exists() {
                fs::remove_dir_all(&quality_dir).unwrap();
            }
            fs::create_dir_all(&quality_dir).unwrap();
        }

        let master_playlist_path = format!("{}/master.m3u", rtsp_dir);
        let mut master_playlist = String::new();

        master_playlist.push_str("#EXTM3U\n");
        master_playlist.push_str("#EXT-X-VERSION:3\n\n");

        for quality in &qualities {
            let playlist_entry = format!(
                "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}x{}\n{}/playlist.m3u8\n",
                quality.bitrate * 1000,
                quality.width,
                quality.height,
                format!("{}p", quality.height)
            );
            master_playlist.push_str(&playlist_entry);
        }

        let _ = fs::write(master_playlist_path, master_playlist);

        qualities
    }
