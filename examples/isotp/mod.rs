use imxrt1060_hal::can::{Frame, Id, CAN};
use imxrt1060_hal::iomuxc::consts::Unsigned;

/// A CAN ISO-TP builder that can build a CAN peripheral
pub struct IsoTPBuilder<M>
where
    M: Unsigned,
{
    can: CAN<M>,
}

impl<M> IsoTPBuilder<M>
where
    M: Unsigned,
{
    pub fn new(
        can: CAN<M>,
    ) -> Self {
        IsoTPBuilder { can }
    }

    pub fn build(self) -> IsoTP<M> {
        IsoTP::new(self.can)
    }
}

pub enum FlowControlType {
    ClearToSend = 0,
    Wait = 1,
    Abort = 2,
}

pub struct IsoTPConfig {
    pub id: Id,
    pub use_padding: bool,
    pub separation_us: bool,
    pub len: u16,
    pub block_size: u16,
    pub flow_control_type: FlowControlType,
    pub separation_time: u16,
    pub padding_value: u8,
}

const RX_BUFFER_LENGTH: usize = 1024;

pub struct IsoTP<M>
where
    M: Unsigned,
{
    pub can: CAN<M>,
    rx_buffer: [u8; RX_BUFFER_LENGTH],
}

impl<M> IsoTP<M>
where
    M: Unsigned,
{
    pub fn new(
        can: CAN<M>,
    ) -> Self {
        Self {
            can,
            rx_buffer: [0x00; RX_BUFFER_LENGTH],
        }
    }

    pub fn write(&mut self, config: &IsoTPConfig, buf: &[u8], size: u16) {
        let mut data: [u8; 8] = [config.padding_value; 8];
        data[0] = ((1 << 4) | size >> 8) as u8;
        data[1] = size as u8;
        data[2..8].copy_from_slice(&buf[0..6]);
        let mut msg: Frame = Frame::new_data(config.id, data);

        self.can.transmit(&msg);

        // let now =
    }
}
