use std::time::Duration;

use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

use crate::{clicker_interface::ClickerInterface, protobuf::protobuf_md_codec::ProtobufMDCodec};

pub struct Clicker {
    io: tokio_util::codec::Framed<tokio_serial::SerialStream, ProtobufMDCodec>,
    timeout: Duration,
}

impl ClickerInterface for Clicker {}

impl Clicker {
    pub fn new<'a>(port: impl Into<std::borrow::Cow<'a, str>>, timeout: Duration) -> Self {
        let port = tokio_serial::new(port, 1500000)
            .open_native_async()
            .unwrap();
        Self {
            io: ProtobufMDCodec.framed(port),
            timeout,
        }
    }
}
