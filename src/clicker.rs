use std::time::Duration;

use futures::{SinkExt, StreamExt};
use rand::RngCore;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;

use crate::{
    clicker_interface::{ClickerInterface, MeasureResult},
    protobuf::{
        self,
        messages::{Mode, OutputReq, Status},
        protobuf_md_codec::ProtobufMDCodec,
        Error,
    },
};

pub struct Clicker {
    io: tokio_util::codec::Framed<tokio_serial::SerialStream, ProtobufMDCodec>,
    timeout: Duration,
}

impl ClickerInterface<Error> for Clicker {
    async fn read(&mut self) -> Result<MeasureResult, Error> {
        self.send_result_request().await?;
        let resp = self.read_responce().await?;

        match Status::try_from(resp.global_status).unwrap() {
            Status::Ok => {
                let output = resp.output.unwrap();
                let mode = Mode::try_from(output.current_mode.unwrap());
                match mode {
                    Ok(Mode::Rk) => Ok(MeasureResult::Rk(output.rk())),
                    Ok(Mode::F) => Ok(MeasureResult::Freq(output.freq())),
                    _ => Err(Error::Protocol(Status::ProtocolError)),
                }
            }
            e => Err(Error::Protocol(e)),
        }
    }
}

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

    pub async fn test(&mut self) -> Result<(), Error> {
        let req = protobuf::new_request();

        self.io.send(req).await?;

        let resp = self.read_responce().await?;
        match Status::try_from(resp.global_status).unwrap() {
            Status::Ok => Ok(()),
            e => Err(Error::Protocol(e)),
        }
    }

    async fn send_result_request(&mut self) -> Result<(), Error> {
        let mut req = protobuf::new_request();
        req.get_output_values = Some(OutputReq {
            get_main_values: Some(protobuf::messages::Empty {}),
            ..Default::default()
        });

        self.io.send(req).await?;
        Ok(())
    }

    async fn read_responce(&mut self) -> Result<protobuf::messages::Response, Error> {
        let res = tokio::time::timeout(self.timeout, self.io.next()).await;
        match res {
            Ok(Some(r)) => r,
            Ok(None) => Err(Error::UnexpectedEndOfStream),
            Err(_) => Err(Error::Timeout),
        }
    }
}
