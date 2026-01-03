//! HackRF One API.
//!
//! To get started take a look at [`HackRfOne::new`].
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

pub use nusb;
use std::io;
use std::io::Read;

use nusb::transfer::{Bulk, ControlIn, ControlOut, ControlType, Direction, In, Recipient};
use nusb::{Interface, MaybeFuture, list_devices};
use std::time::Duration;

#[cfg(feature = "num-complex")]
pub use num_complex;

/// HackRF USB vendor ID.
const HACKRF_USB_VID: u16 = 0x1D50;
/// HackRF One USB product ID.
const HACKRF_ONE_USB_PID: u16 = 0x6089;

#[allow(dead_code)]
#[repr(u8)]
enum Request {
    SetTransceiverMode = 1,
    Max2837Write = 2,
    Max2837Read = 3,
    Si5351CWrite = 4,
    Si5351CRead = 5,
    SampleRateSet = 6,
    BasebandFilterBandwidthSet = 7,
    Rffc5071Write = 8,
    Rffc5071Read = 9,
    SpiflashErase = 10,
    SpiflashWrite = 11,
    SpiflashRead = 12,
    BoardIdRead = 14,
    VersionStringRead = 15,
    SetFreq = 16,
    AmpEnable = 17,
    BoardPartidSerialnoRead = 18,
    SetLnaGain = 19,
    SetVgaGain = 20,
    SetTxvgaGain = 21,
    AntennaEnable = 23,
    SetFreqExplicit = 24,
    UsbWcidVendorReq = 25,
    InitSweep = 26,
    OperacakeGetBoards = 27,
    OperacakeSetPorts = 28,
    SetHwSyncMode = 29,
    Reset = 30,
    OperacakeSetRanges = 31,
    ClkoutEnable = 32,
    SpiflashStatus = 33,
    SpiflashClearStatus = 34,
    OperacakeGpioTest = 35,
    CpldChecksum = 36,
    UiEnable = 37,
}

impl From<Request> for u8 {
    fn from(r: Request) -> Self {
        r as u8
    }
}

#[allow(dead_code)]
#[repr(u8)]
enum TransceiverMode {
    Off = 0,
    Receive = 1,
    Transmit = 2,
    Ss = 3,
    CpldUpdate = 4,
    RxSweep = 5,
}

impl From<TransceiverMode> for u8 {
    fn from(tm: TransceiverMode) -> Self {
        tm as u8
    }
}

impl From<TransceiverMode> for u16 {
    fn from(tm: TransceiverMode) -> Self {
        tm as u16
    }
}

/// HackRF One errors.
#[derive(Debug)]
pub enum Error {
    /// USB error.
    Usb(nusb::Error),
    /// USB Connection Errors
    UsbTransfer(nusb::transfer::TransferError),
    /// IO Error
    IO(io::Error),
    /// Failed to transfer all bytes in a control transfer.
    CtrlTransfer {
        /// Control transfer direction.
        dir: Direction,
        /// Actual amount of bytes transferred.
        actual: usize,
        /// Excepted number of bytes transferred.
        expected: usize,
    },
    /// An API call is not supported by your hardware.
    ///
    /// Try updating the firmware on your device.
    Version {
        /// Current device version.
        device: Version,
        /// Minimum version required.
        min: Version,
    },
    /// A provided argument was out of range.
    Argument,
}

impl From<nusb::Error> for Error {
    fn from(e: nusb::Error) -> Self {
        Error::Usb(e)
    }
}

impl From<nusb::transfer::TransferError> for Error {
    fn from(e: nusb::transfer::TransferError) -> Self {
        Error::UsbTransfer(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Version used to denote the parts of BCD
pub struct Version {
    /// Major version XX.0.0
    pub major: u8,
    /// Minor version 00.X.0
    pub minor: u8,
    /// Sub Minor version 00.0.X
    pub sub_minor: u8,
}

impl Version {
    fn from_bcd(raw: u16) -> Self {
        // 0xJJMN JJ major, M minor, N sub-minor
        // Binary Coded Decimal
        let major0: u8 = ((raw & 0xF000) >> 12) as u8;
        let major1: u8 = ((raw & 0x0F00) >> 8) as u8;

        let minor: u8 = ((raw & 0x00F0) >> 4) as u8;

        let sub_minor: u8 = (raw & 0x000F) as u8;

        Self {
            major: (major0 * 10) + major1,
            minor,
            sub_minor,
        }
    }
}

#[cfg(test)]
mod version_bcd {
    use super::Version;

    #[test]
    fn from_bcd() {
        assert_eq!(
            Version::from_bcd(0x1234),
            Version {
                major: 12,
                minor: 3,
                sub_minor: 4
            }
        );
        assert_eq!(
            Version::from_bcd(0x4321),
            Version {
                major: 43,
                minor: 2,
                sub_minor: 1
            }
        );
        assert_eq!(
            Version::from_bcd(0x0200),
            Version {
                major: 2,
                minor: 0,
                sub_minor: 0
            }
        );
        assert_eq!(
            Version::from_bcd(0x0110),
            Version {
                major: 1,
                minor: 1,
                sub_minor: 0
            }
        );
    }
}

impl std::error::Error for Error {}

/// Typestate for RX mode.
#[derive(Debug)]
pub struct RxMode;

/// Typestate for an unknown mode.
#[derive(Debug)]
pub struct UnknownMode;

/// HackRF One software defined radio.
pub struct HackRfOne<MODE> {
    dh: nusb::Device,
    desc: nusb::descriptors::DeviceDescriptor,
    interface: Interface,
    #[allow(dead_code)]
    mode: MODE,
    timeout: Duration,
}

impl HackRfOne<UnknownMode> {
    /// Open a new HackRF One.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// ```
    #[must_use]
    pub fn new() -> Option<HackRfOne<UnknownMode>> {
        let Ok(devices) = list_devices().wait() else {
            return None;
        };

        for device in devices {
            if device.vendor_id() == HACKRF_USB_VID && device.product_id() == HACKRF_ONE_USB_PID {
                match device.open().wait() {
                    Ok(handle) => {
                        let Ok(interface) = handle.claim_interface(0).wait() else {
                            return None;
                        };

                        return Some(HackRfOne {
                            desc: handle.device_descriptor(),
                            dh: handle,
                            interface,
                            mode: UnknownMode,
                            timeout: Duration::from_secs(1),
                        });
                    }
                    Err(_) => continue,
                }
            }
        }

        None
    }
}

impl<MODE> HackRfOne<MODE> {
    fn read_control<const N: usize>(
        &self,
        request: Request,
        value: u16,
        index: u16,
    ) -> Result<[u8; N], Error> {
        let buf = self
            .interface
            .control_in(
                ControlIn {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Device,
                    request: request.into(),
                    value,
                    index,
                    length: N as u16,
                },
                self.timeout,
            )
            .wait()?;

        if N == buf.len() {
            Ok(<[u8; N]>::try_from(buf).expect("This should never happen"))
        } else {
            Err(Error::CtrlTransfer {
                dir: Direction::In,
                actual: buf.len(),
                expected: N,
            })
        }
    }

    fn write_control(
        &mut self,
        request: Request,
        value: u16,
        index: u16,
        buf: &[u8],
    ) -> Result<(), Error> {
        self.interface
            .control_out(
                ControlOut {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Device,
                    request: request.into(),
                    value,
                    index,
                    data: buf,
                },
                self.timeout,
            )
            .wait()?;

        Ok(())
    }

    fn check_api_version(&self, min: Version) -> Result<(), Error> {
        fn version_to_u32(v: &Version) -> u32 {
            (u32::from(v.major) << 16) | (u32::from(v.minor) << 8) | u32::from(v.sub_minor)
        }

        let v: Version = self.device_version();
        let v_cmp: u32 = version_to_u32(&v);
        let min_cmp: u32 = version_to_u32(&min);

        if v_cmp >= min_cmp {
            Ok(())
        } else {
            Err(Error::Version { device: v, min })
        }
    }

    /// Get the device version from the USB descriptor.
    ///
    /// The HackRF C API calls the equivalent of this function
    /// `hackrf_usb_api_version_read`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode, Version};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// assert_eq!(
    ///     radio.device_version(),
    ///     Version {
    ///         major: 1,
    ///         minor: 0,
    ///         sub_minor: 4
    ///     }
    /// );
    /// ```
    pub fn device_version(&self) -> Version {
        Version::from_bcd(self.desc.device_version())
    }

    /// Set the timeout for USB transfers.
    ///
    /// # Example
    ///
    /// Set a 100ms timeout.
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    /// use std::time::Duration;
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// radio.set_timeout(Duration::from_millis(100))
    /// ```
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout = duration;
    }

    /// Read the board ID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// assert_eq!(radio.board_id()?, 0x02);
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn board_id(&self) -> Result<u8, Error> {
        let data: [u8; 1] = self.read_control(Request::BoardIdRead, 0, 0)?;
        Ok(data[0])
    }

    /// Read the firmware version.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// assert_eq!(radio.version()?, "2021.03.1");
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn version(&self) -> Result<String, Error> {
        let buf = self
            .interface
            .control_in(
                ControlIn {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Device,
                    request: Request::VersionStringRead.into(),
                    value: 0,
                    index: 0,
                    length: 16,
                },
                self.timeout,
            )
            .wait()?;

        Ok(String::from_utf8_lossy(&buf[0..16]).into())
    }

    /// Set the center frequency.
    ///
    /// # Example
    ///
    /// Set the frequency to 915MHz.
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// radio.set_freq(915_000_000)?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn set_freq(&mut self, hz: u64) -> Result<(), Error> {
        let buf: [u8; 8] = freq_params(hz);
        self.write_control(Request::SetFreq, 0, 0, &buf)
    }

    /// Enable the RX/TX RF amplifier.
    ///
    /// In GNU radio this is used as the RF gain, where a value of 0 dB is off,
    /// and a value of 14 dB is on.
    ///
    /// # Example
    ///
    /// Disable the amplifier.
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// radio.set_amp_enable(false)?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn set_amp_enable(&mut self, en: bool) -> Result<(), Error> {
        self.write_control(Request::AmpEnable, en.into(), 0, &[])
    }

    /// Set the baseband filter bandwidth.
    ///
    /// This is automatically set when the sample rate is changed with
    /// [`set_sample_rate`].
    ///
    /// # Example
    ///
    /// Set the filter bandwidth to 70% of the sample rate.
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// const SAMPLE_HZ: u32 = 20_000_000;
    /// const SAMPLE_DIV: u32 = 2;
    /// const FILTER_BW: u32 = (0.7 * (SAMPLE_HZ as f32) / (SAMPLE_DIV as f32)) as u32;
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// radio.set_sample_rate(SAMPLE_HZ, SAMPLE_DIV)?;
    /// radio.set_baseband_filter_bandwidth(FILTER_BW)?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    ///
    /// [`set_sample_rate`]: crate::HackRfOne::set_sample_rate
    pub fn set_baseband_filter_bandwidth(&mut self, hz: u32) -> Result<(), Error> {
        self.write_control(
            Request::BasebandFilterBandwidthSet,
            (hz & 0xFFFF) as u16,
            (hz >> 16) as u16,
            &[],
        )
    }

    /// Set the sample rate.
    ///
    /// For anti-aliasing, the baseband filter bandwidth is automatically set to
    /// the widest available setting that is no more than 75% of the sample rate.
    /// This happens every time the sample rate is set.
    /// If you want to override the baseband filter selection, you must do so
    /// after setting the sample rate.
    ///
    /// Limits are 8MHz - 20MHz.
    /// Preferred rates are 8, 10, 12.5, 16, 20MHz due to less jitter.
    ///
    /// # Example
    ///
    /// Set the sample rate to 10 MHz.
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// radio.set_sample_rate(20_000_000, 2)?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn set_sample_rate(&mut self, hz: u32, div: u32) -> Result<(), Error> {
        let hz: u32 = hz.to_le();
        let div: u32 = div.to_le();
        let buf: [u8; 8] = [
            (hz & 0xFF) as u8,
            ((hz >> 8) & 0xFF) as u8,
            ((hz >> 16) & 0xFF) as u8,
            ((hz >> 24) & 0xFF) as u8,
            (div & 0xFF) as u8,
            ((div >> 8) & 0xFF) as u8,
            ((div >> 16) & 0xFF) as u8,
            ((div >> 24) & 0xFF) as u8,
        ];
        self.write_control(Request::SampleRateSet, 0, 0, &buf)?;
        self.set_baseband_filter_bandwidth((0.75 * (hz as f32) / (div as f32)) as u32)
    }

    /// Set the LNA (low noise amplifier) gain.
    ///
    /// Range 0 to 40dB in 8dB steps.
    ///
    /// This is also known as the IF gain.
    ///
    /// # Example
    ///
    /// Set the LNA gain to 16 dB (generally a reasonable gain to start with).
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// radio.set_lna_gain(16)?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn set_lna_gain(&mut self, gain: u16) -> Result<(), Error> {
        if gain > 40 {
            Err(Error::Argument)
        } else {
            let buf: [u8; 1] = self.read_control(Request::SetLnaGain, 0, gain & !0x07)?;
            if buf[0] == 0 {
                Err(Error::Argument)
            } else {
                Ok(())
            }
        }
    }

    /// Set the VGA (variable gain amplifier) gain.
    ///
    /// Range 0 to 62dB in 2dB steps.
    ///
    /// This is also known as the baseband (BB) gain.
    ///
    /// # Example
    ///
    /// Set the VGA gain to 16 dB (generally a reasonable gain to start with).
    ///
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// radio.set_vga_gain(16)?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn set_vga_gain(&mut self, gain: u16) -> Result<(), Error> {
        if gain > 62 {
            Err(Error::Argument)
        } else {
            let buf: [u8; 1] = self.read_control(Request::SetVgaGain, 0, gain & !0b1)?;
            if buf[0] == 0 {
                Err(Error::Argument)
            } else {
                Ok(())
            }
        }
    }

    /// Set the transmit VGA gain.
    ///
    /// Range 0 to 47dB in 1db steps.
    pub fn set_txvga_gain(&mut self, gain: u16) -> Result<(), Error> {
        if gain > 47 {
            Err(Error::Argument)
        } else {
            let buf: [u8; 1] = self.read_control(Request::SetTxvgaGain, 0, gain)?;
            if buf[0] == 0 {
                Err(Error::Argument)
            } else {
                Ok(())
            }
        }
    }

    /// Antenna power port control.
    ///
    /// The source docs are a little lacking in terms of explanations here.
    pub fn set_antenna_enable(&mut self, value: u8) -> Result<(), Error> {
        self.write_control(Request::AntennaEnable, value.into(), 0, &[])
    }

    /// CLKOUT enable.
    ///
    /// The source docs are a little lacking in terms of explanations here.
    pub fn set_clkout_enable(&mut self, en: bool) -> Result<(), Error> {
        self.check_api_version(Version::from_bcd(0x0103))?;
        self.write_control(Request::ClkoutEnable, en.into(), 0, &[])
    }

    /// Reset the HackRF radio.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// let mut radio: HackRfOne<UnknownMode> = radio.reset()?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn reset(mut self) -> Result<HackRfOne<UnknownMode>, Error> {
        self.check_api_version(Version::from_bcd(0x0102))?;
        self.write_control(Request::Reset, 0, 0, &[])?;
        Ok(HackRfOne {
            dh: self.dh,
            desc: self.desc,
            interface: self.interface,
            mode: UnknownMode,
            timeout: self.timeout,
        })
    }

    fn set_transceiver_mode(&mut self, mode: TransceiverMode) -> Result<(), Error> {
        self.write_control(Request::SetTransceiverMode, mode.into(), 0, &[])
    }

    /// Change the radio mode to RX.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, RxMode, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// let mut radio: HackRfOne<RxMode> = radio.into_rx_mode()?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn into_rx_mode(mut self) -> Result<HackRfOne<RxMode>, Error> {
        self.set_transceiver_mode(TransceiverMode::Receive)?;
        Ok(HackRfOne {
            dh: self.dh,
            desc: self.desc,
            interface: self.interface,
            mode: RxMode,
            timeout: self.timeout,
        })
    }
}

impl HackRfOne<RxMode> {
    /// Receive data from the radio.
    ///
    /// This uses a bulk transfer to get one MTU (maximum transmission unit)
    /// of data in a single shot.  The data format is pairs of signed 8-bit IQ.
    /// Use the [`iq_to_cplx_i8`] or [`iq_to_cplx_f32`] helpers to convert the
    /// data to a more manageable format.
    ///
    /// Unlike `libhackrf` this does not spawn a sampling thread.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, RxMode, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// let mut radio: HackRfOne<RxMode> = radio.into_rx_mode()?;
    /// let data: Vec<u8> = radio.rx()?;
    /// radio.stop_rx()?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    ///
    /// [`iq_to_cplx_i8`]: crate::iq_to_cplx_i8
    /// [`iq_to_cplx_f32`]: crate::iq_to_cplx_f32
    #[cfg_attr(not(feature = "num-complex"), allow(rustdoc::broken_intra_doc_links))]
    pub fn rx(&mut self) -> Result<Vec<u8>, Error> {
        const ENDPOINT: u8 = 0x81;
        const MTU: usize = 128 * 1024;
        let mut buf: Vec<u8> = vec![0; MTU];
        let mut reader = self.interface.endpoint::<Bulk, In>(ENDPOINT)?.reader(MTU);
        let n = reader.read(buf.as_mut_slice())?;
        buf.truncate(n);
        Ok(buf)
    }

    /// Stop receiving.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hackrfone::{HackRfOne, RxMode, UnknownMode};
    ///
    /// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
    /// let mut radio: HackRfOne<RxMode> = radio.into_rx_mode()?;
    /// let data: Vec<u8> = radio.rx()?;
    /// radio.stop_rx()?;
    /// # Ok::<(), hackrfone::Error>(())
    /// ```
    pub fn stop_rx(mut self) -> Result<HackRfOne<UnknownMode>, Error> {
        self.set_transceiver_mode(TransceiverMode::Off)?;
        Ok(HackRfOne {
            dh: self.dh,
            desc: self.desc,
            interface: self.interface,
            mode: UnknownMode,
            timeout: self.timeout,
        })
    }
}

// Helper for set_freq
fn freq_params(hz: u64) -> [u8; 8] {
    const MHZ: u64 = 1_000_000;

    let l_freq_mhz: u32 = u32::try_from(hz / MHZ).unwrap_or(u32::MAX).to_le();
    let l_freq_hz: u32 = u32::try_from(hz - u64::from(l_freq_mhz) * MHZ)
        .unwrap_or(u32::MAX)
        .to_le();

    [
        (l_freq_mhz & 0xFF) as u8,
        ((l_freq_mhz >> 8) & 0xFF) as u8,
        ((l_freq_mhz >> 16) & 0xFF) as u8,
        ((l_freq_mhz >> 24) & 0xFF) as u8,
        (l_freq_hz & 0xFF) as u8,
        ((l_freq_hz >> 8) & 0xFF) as u8,
        ((l_freq_hz >> 16) & 0xFF) as u8,
        ((l_freq_hz >> 24) & 0xFF) as u8,
    ]
}

#[cfg(test)]
mod freq_params {
    use super::freq_params;

    #[test]
    fn nominal() {
        assert_eq!(freq_params(915_000_000), [0x93, 0x03, 0, 0, 0, 0, 0, 0]);
        assert_eq!(freq_params(915_000_001), [0x93, 0x03, 0, 0, 1, 0, 0, 0]);
        assert_eq!(
            freq_params(123456789),
            [0x7B, 0, 0, 0, 0x55, 0xF8, 0x06, 0x00]
        );
    }

    #[test]
    fn min() {
        assert_eq!(freq_params(0), [0; 8]);
    }

    #[test]
    fn max() {
        assert_eq!(freq_params(u64::MAX), [0xFF; 8]);
    }
}

/// Convert an IQ sample pair to a complex number.
///
/// # Example
///
/// Post-processing sample data.
///
/// ```no_run
/// use hackrfone::{HackRfOne, RxMode, UnknownMode, iq_to_cplx_i8};
///
/// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
/// let mut radio: HackRfOne<RxMode> = radio.into_rx_mode()?;
/// let data: Vec<u8> = radio.rx()?;
/// radio.stop_rx()?;
///
/// for iq in data.chunks_exact(2) {
///     let cplx: num_complex::Complex<i8> = iq_to_cplx_i8(iq[0], iq[1]);
///     // .. do whatever you want with cplx here
/// }
///
/// # Ok::<(), hackrfone::Error>(())
/// ```
///
/// Guide level explanation.
///
/// ```
/// use hackrfone::iq_to_cplx_i8;
/// use num_complex::Complex;
///
/// assert_eq!(iq_to_cplx_i8(255, 1), Complex::new(-1, 1));
/// ```
#[cfg(feature = "num-complex")]
pub fn iq_to_cplx_i8(i: u8, q: u8) -> num_complex::Complex<i8> {
    num_complex::Complex::new(i as i8, q as i8)
}

/// Convert an IQ sample pair to a floating point complex number.
///
/// Generally you will want to use [`iq_to_cplx_i8`] for storing or transfering
/// data because the samples are 2-bytes in the native i8, vs 8-bytes in f32.
///
/// Floats are easier to work with for running samples through digital signal
/// processing algorithms (e.g. discrete fourier transform) where the i8 can
/// easily saturate.
///
/// # Example
///
/// Post-processing sample data.
///
/// ```no_run
/// use hackrfone::{HackRfOne, RxMode, UnknownMode, iq_to_cplx_f32};
///
/// let mut radio: HackRfOne<UnknownMode> = HackRfOne::new().unwrap();
/// let mut radio: HackRfOne<RxMode> = radio.into_rx_mode()?;
/// let data: Vec<u8> = radio.rx()?;
/// radio.stop_rx()?;
///
/// for iq in data.chunks_exact(2) {
///     let cplx: num_complex::Complex<f32> = iq_to_cplx_f32(iq[0], iq[1]);
///     // .. do whatever you want with cplx here
/// }
///
/// # Ok::<(), hackrfone::Error>(())
/// ```
///
/// Guide level explanation.
///
/// ```
/// use hackrfone::iq_to_cplx_f32;
/// use num_complex::Complex;
///
/// assert_eq!(iq_to_cplx_f32(255, 1), Complex::new(-1.0, 1.0));
/// ```
#[cfg(feature = "num-complex")]
pub fn iq_to_cplx_f32(i: u8, q: u8) -> num_complex::Complex<f32> {
    num_complex::Complex::new(i as i8 as f32, q as i8 as f32)
}
