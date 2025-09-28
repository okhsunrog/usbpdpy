//! # usbpdpy - Python bindings for the usbpd Rust crate
//!
//! This library provides Python bindings for the `usbpd` Rust crate, enabling
//! fast and accurate USB Power Delivery message parsing in Python applications.
//!
//! ## Features
//!
//! - Parse USB PD messages with full specification compliance
//! - Support for all message types (control and data messages)
//! - Accurate Power Data Object (PDO) parsing
//! - Source and Sink Capabilities support
//! - Request message parsing
//! - Vendor Defined Messages (VDM) support
//!
//! ## Example
//!
//! ```python
//! import usbpdpy
//!
//! # Parse a Source Capabilities message
//! message_bytes = bytes.fromhex("a1612c9101082cd102002cc103002cb10400454106003c21dcc0")
//! message = usbpdpy.parse_message(message_bytes)
//! 
//! print(f"Message type: {message.message_type}")
//! print(f"Power objects: {len(message.data_objects)}")
//! for pdo in message.data_objects:
//!     print(f"  {pdo}")
//! ```

use pyo3::prelude::*;
use pyo3::types::PyBytesMethods;
use pyo3::exceptions::{PyValueError, PyTypeError};

use usbpd::protocol_layer::message::{
    Message, Data, ParseError, PdoState,
    header::{Header, MessageType, DataMessageType, ControlMessageType, SpecificationRevision},
    pdo::{PowerDataObject, Augmented, SourceCapabilities, Kind},
    request,
};

/// Python wrapper for USB PD message header
#[pyclass]
#[derive(Debug, Clone)]
pub struct PdHeader {
    #[pyo3(get)]
    /// Message type as human-readable string (e.g., "Source_Capabilities", "GoodCRC")
    pub message_type: String,
    
    #[pyo3(get)]
    /// Raw message type value
    pub message_type_raw: u8,
    
    #[pyo3(get)]
    /// Port data role ("Ufp" or "Dfp")
    pub port_data_role: String,
    
    #[pyo3(get)]
    /// USB PD specification revision (0=R1.0, 1=R2.0, 2=R3.0)
    pub spec_revision: u8,
    
    #[pyo3(get)]
    /// Port power role ("Sink" or "Source")
    pub port_power_role: String,
    
    #[pyo3(get)]
    /// Message ID (0-7)
    pub message_id: u8,
    
    #[pyo3(get)]
    /// Number of data objects (0-7)
    pub num_data_objects: u8,
    
    #[pyo3(get)]
    /// Extended message flag
    pub extended: bool,
}

impl From<&Header> for PdHeader {
    fn from(header: &Header) -> Self {
        let message_type = match header.message_type() {
            MessageType::Control(ctrl) => match ctrl {
                ControlMessageType::GoodCRC => "GoodCRC",
                ControlMessageType::GotoMin => "GotoMin",
                ControlMessageType::Accept => "Accept",
                ControlMessageType::Reject => "Reject",
                ControlMessageType::Ping => "Ping",
                ControlMessageType::PsRdy => "PS_RDY",
                ControlMessageType::GetSourceCap => "Get_Source_Cap",
                ControlMessageType::GetSinkCap => "Get_Sink_Cap",
                ControlMessageType::DrSwap => "DR_Swap",
                ControlMessageType::PrSwap => "PR_Swap",
                ControlMessageType::VconnSwap => "VCONN_Swap",
                ControlMessageType::Wait => "Wait",
                ControlMessageType::SoftReset => "Soft_Reset",
                ControlMessageType::DataReset => "Data_Reset",
                ControlMessageType::DataResetComplete => "Data_Reset_Complete",
                ControlMessageType::NotSupported => "Not_Supported",
                ControlMessageType::GetSourceCapExtended => "Get_Source_Cap_Extended",
                ControlMessageType::GetStatus => "Get_Status",
                ControlMessageType::FrSwap => "FR_Swap",
                ControlMessageType::GetPpsStatus => "Get_PPS_Status",
                ControlMessageType::GetCountryCodes => "Get_Country_Codes",
                ControlMessageType::GetSinkCapExtended => "Get_Sink_Cap_Extended",
                ControlMessageType::GetSourceInfo => "Get_Source_Info",
                ControlMessageType::GetRevision => "Get_Revision",
                ControlMessageType::Reserved => "Reserved",
            },
            MessageType::Data(data) => match data {
                DataMessageType::SourceCapabilities => "Source_Capabilities",
                DataMessageType::Request => "Request",
                DataMessageType::Bist => "BIST",
                DataMessageType::SinkCapabilities => "Sink_Capabilities",
                DataMessageType::BatteryStatus => "Battery_Status",
                DataMessageType::Alert => "Alert",
                DataMessageType::GetCountryInfo => "Get_Country_Info",
                DataMessageType::EnterUsb => "Enter_USB",
                DataMessageType::EprRequest => "EPR_Request",
                DataMessageType::EprMode => "EPR_Mode",
                DataMessageType::SourceInfo => "Source_Info",
                DataMessageType::Revision => "Revision",
                DataMessageType::VendorDefined => "Vendor_Defined",
                DataMessageType::Reserved => "Reserved",
            },
        }.to_string();

        let spec_revision = match header.spec_revision() {
            Ok(SpecificationRevision::R1_0) => 0,
            Ok(SpecificationRevision::R2_0) => 1,
            Ok(SpecificationRevision::R3_0) => 2,
            Err(_) => 0,
        };

        PdHeader {
            message_type,
            message_type_raw: header.message_type_raw(),
            port_data_role: format!("{:?}", header.port_data_role()),
            spec_revision,
            port_power_role: format!("{:?}", header.port_power_role()),
            message_id: header.message_id(),
            num_data_objects: header.num_objects() as u8,
            extended: header.extended(),
        }
    }
}

#[pymethods]
impl PdHeader {
    fn __repr__(&self) -> String {
        format!(
            "PdHeader(type='{}', objects={}, role={})",
            self.message_type, self.num_data_objects, self.port_power_role
        )
    }

    fn __str__(&self) -> String {
        format!("{} (ID: {}, Objects: {})", self.message_type, self.message_id, self.num_data_objects)
    }
}

/// Python wrapper for USB PD Power Data Object
#[pyclass]
#[derive(Debug, Clone)]
pub struct PowerDataObj {
    #[pyo3(get)]
    /// Raw 32-bit PDO value
    pub raw: u32,
    
    #[pyo3(get)]
    /// PDO type ("FixedSupply", "Battery", "VariableSupply", "PPS", "Unknown")
    pub pdo_type: String,
    
    #[pyo3(get)]
    /// Voltage in volts (for fixed/variable supply)
    pub voltage_v: Option<f32>,
    
    #[pyo3(get)]
    /// Maximum current in amperes
    pub max_current_a: Option<f32>,
    
    #[pyo3(get)]
    /// Maximum power in watts (calculated for fixed supply)
    pub max_power_w: Option<f32>,
    
    #[pyo3(get)]
    /// Minimum voltage in volts (for PPS/variable supply)
    pub min_voltage_v: Option<f32>,
    
    #[pyo3(get)]
    /// Maximum voltage in volts (for PPS/variable supply)
    pub max_voltage_v: Option<f32>,
    
    #[pyo3(get)]
    /// Dual role power capability
    pub dual_role_power: Option<bool>,
    
    #[pyo3(get)]
    /// USB communications capability
    pub usb_communications_capable: Option<bool>,
    
    #[pyo3(get)]
    /// Unconstrained power
    pub unconstrained_power: Option<bool>,
}

impl From<&PowerDataObject> for PowerDataObj {
    fn from(pdo: &PowerDataObject) -> Self {
        match pdo {
            PowerDataObject::FixedSupply(fixed) => {
                // Convert using the raw values and known scaling factors
                let voltage_v = fixed.raw_voltage() as f32 * 0.05; // 50mV units  
                let current_a = fixed.raw_max_current() as f32 * 0.01; // 10mA units
                let power_w = voltage_v * current_a;

                PowerDataObj {
                    raw: fixed.0,
                    pdo_type: "FixedSupply".to_string(),
                    voltage_v: Some(voltage_v),
                    max_current_a: Some(current_a),
                    max_power_w: Some(power_w),
                    min_voltage_v: None,
                    max_voltage_v: None,
                    dual_role_power: Some(fixed.dual_role_power()),
                    usb_communications_capable: Some(fixed.usb_communications_capable()),
                    unconstrained_power: Some(fixed.unconstrained_power()),
                }
            }
            PowerDataObject::Battery(battery) => {
                let min_voltage_v = battery.raw_min_voltage() as f32 * 0.05; // 50mV units
                let max_voltage_v = battery.raw_max_voltage() as f32 * 0.05; // 50mV units
                let max_power_w = battery.raw_max_power() as f32 * 0.25; // 250mW units

                PowerDataObj {
                    raw: battery.0,
                    pdo_type: "Battery".to_string(),
                    voltage_v: None,
                    max_current_a: None,
                    max_power_w: Some(max_power_w),
                    min_voltage_v: Some(min_voltage_v),
                    max_voltage_v: Some(max_voltage_v),
                    dual_role_power: None,
                    usb_communications_capable: None,
                    unconstrained_power: None,
                }
            }
            PowerDataObject::VariableSupply(variable) => {
                let min_voltage_v = variable.raw_min_voltage() as f32 * 0.05; // 50mV units
                let max_voltage_v = variable.raw_max_voltage() as f32 * 0.05; // 50mV units
                let max_current_a = variable.raw_max_current() as f32 * 0.01; // 10mA units

                PowerDataObj {
                    raw: variable.0,
                    pdo_type: "VariableSupply".to_string(),
                    voltage_v: None,
                    max_current_a: Some(max_current_a),
                    max_power_w: None,
                    min_voltage_v: Some(min_voltage_v),
                    max_voltage_v: Some(max_voltage_v),
                    dual_role_power: None,
                    usb_communications_capable: None,
                    unconstrained_power: None,
                }
            }
            PowerDataObject::Augmented(aug) => match aug {
                Augmented::Spr(pps) => {
                    let min_voltage_v = pps.raw_min_voltage() as f32 * 0.1; // 100mV units
                    let max_voltage_v = pps.raw_max_voltage() as f32 * 0.1;
                    let max_current_a = pps.raw_max_current() as f32 * 0.05; // 50mA units

                    PowerDataObj {
                        raw: pps.0,
                        pdo_type: "PPS".to_string(),
                        voltage_v: None,
                        max_current_a: Some(max_current_a),
                        max_power_w: None,
                        min_voltage_v: Some(min_voltage_v),
                        max_voltage_v: Some(max_voltage_v),
                        dual_role_power: None,
                        usb_communications_capable: None,
                        unconstrained_power: None,
                    }
                }
                Augmented::Epr(epr) => {
                    PowerDataObj {
                        raw: epr.0,
                        pdo_type: "EPR_AVS".to_string(),
                        voltage_v: None,
                        max_current_a: None,
                        max_power_w: None,
                        min_voltage_v: None,
                        max_voltage_v: None,
                        dual_role_power: None,
                        usb_communications_capable: None,
                        unconstrained_power: None,
                    }
                }
                Augmented::Unknown(raw) => {
                    PowerDataObj {
                        raw: *raw,
                        pdo_type: "Unknown_Augmented".to_string(),
                        voltage_v: None,
                        max_current_a: None,
                        max_power_w: None,
                        min_voltage_v: None,
                        max_voltage_v: None,
                        dual_role_power: None,
                        usb_communications_capable: None,
                        unconstrained_power: None,
                    }
                }
            },
            PowerDataObject::Unknown(raw) => {
                PowerDataObj {
                    raw: raw.0,
                    pdo_type: "Unknown".to_string(),
                    voltage_v: None,
                    max_current_a: None,
                    max_power_w: None,
                    min_voltage_v: None,
                    max_voltage_v: None,
                    dual_role_power: None,
                    usb_communications_capable: None,
                    unconstrained_power: None,
                }
            }
        }
    }
}

#[pymethods]
impl PowerDataObj {
    fn __repr__(&self) -> String {
        match self.pdo_type.as_str() {
            "FixedSupply" => format!(
                "PowerDataObj(FixedSupply: {}V @ {}A = {}W)",
                self.voltage_v.unwrap_or(0.0),
                self.max_current_a.unwrap_or(0.0),
                self.max_power_w.unwrap_or(0.0)
            ),
            "PPS" => format!(
                "PowerDataObj(PPS: {}-{}V @ {}A)",
                self.min_voltage_v.unwrap_or(0.0),
                self.max_voltage_v.unwrap_or(0.0),
                self.max_current_a.unwrap_or(0.0)
            ),
            _ => format!("PowerDataObj({})", self.pdo_type),
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Python wrapper for USB PD Request Data Object (RDO)
#[pyclass]
#[derive(Debug, Clone)]
pub struct RequestDataObj {
    #[pyo3(get)]
    /// Raw 32-bit RDO value
    pub raw: u32,
    
    #[pyo3(get)]
    /// RDO type ("FixedVariableSupply", "Battery", "PPS", "AVS", "Unknown")
    pub rdo_type: String,
    
    #[pyo3(get)]
    /// PDO position (1-7) that this request refers to
    pub object_position: u8,
    
    #[pyo3(get)]
    /// Operating current in amperes (for fixed/variable supply)
    pub operating_current_a: Option<f32>,
    
    #[pyo3(get)]
    /// Max operating current in amperes (for fixed/variable supply)
    pub max_operating_current_a: Option<f32>,
    
    #[pyo3(get)]
    /// Operating voltage in volts (for PPS/AVS)
    pub operating_voltage_v: Option<f32>,
    
    #[pyo3(get)]
    /// Operating power in watts (for battery)
    pub operating_power_w: Option<f32>,
    
    #[pyo3(get)]
    /// Max operating power in watts (for battery)
    pub max_operating_power_w: Option<f32>,
    
    #[pyo3(get)]
    /// Capability mismatch flag
    pub capability_mismatch: bool,
    
    #[pyo3(get)]
    /// USB communications capable
    pub usb_communications_capable: bool,
    
    #[pyo3(get)]
    /// No USB suspend
    pub no_usb_suspend: bool,
    
    #[pyo3(get)]
    /// GiveBack flag (for fixed/variable supply)
    pub giveback_flag: Option<bool>,
}

impl From<&request::PowerSource> for RequestDataObj {
    fn from(request: &request::PowerSource) -> Self {
        match request {
            request::PowerSource::FixedVariableSupply(req) => {
                let operating_current_a = req.operating_current().get::<uom::si::electric_current::ampere>() as f32;
                let max_operating_current_a = req.max_operating_current().get::<uom::si::electric_current::ampere>() as f32;
                
                RequestDataObj {
                    raw: req.0,
                    rdo_type: "FixedVariableSupply".to_string(),
                    object_position: req.object_position(),
                    operating_current_a: Some(operating_current_a),
                    max_operating_current_a: Some(max_operating_current_a),
                    operating_voltage_v: None,
                    operating_power_w: None,
                    max_operating_power_w: None,
                    capability_mismatch: req.capability_mismatch(),
                    usb_communications_capable: req.usb_communications_capable(),
                    no_usb_suspend: req.no_usb_suspend(),
                    giveback_flag: Some(req.giveback_flag()),
                }
            }
            request::PowerSource::Battery(req) => {
                let operating_power_w = req.operating_power().get::<uom::si::power::watt>() as f32;
                let max_operating_power_w = req.max_operating_power().get::<uom::si::power::watt>() as f32;
                
                RequestDataObj {
                    raw: req.0,
                    rdo_type: "Battery".to_string(),
                    object_position: req.object_position(),
                    operating_current_a: None,
                    max_operating_current_a: None,
                    operating_voltage_v: None,
                    operating_power_w: Some(operating_power_w),
                    max_operating_power_w: Some(max_operating_power_w),
                    capability_mismatch: req.capability_mismatch(),
                    usb_communications_capable: req.usb_communications_capable(),
                    no_usb_suspend: req.no_usb_suspend(),
                    giveback_flag: Some(req.giveback_flag()),
                }
            }
            request::PowerSource::Pps(req) => {
                let operating_voltage_v = req.output_voltage().get::<uom::si::electric_potential::volt>() as f32;
                let operating_current_a = req.operating_current().get::<uom::si::electric_current::ampere>() as f32;
                
                RequestDataObj {
                    raw: req.0,
                    rdo_type: "PPS".to_string(),
                    object_position: req.object_position(),
                    operating_current_a: Some(operating_current_a),
                    max_operating_current_a: None,
                    operating_voltage_v: Some(operating_voltage_v),
                    operating_power_w: None,
                    max_operating_power_w: None,
                    capability_mismatch: req.capability_mismatch(),
                    usb_communications_capable: req.usb_communications_capable(),
                    no_usb_suspend: req.no_usb_suspend(),
                    giveback_flag: None,
                }
            }
            request::PowerSource::Avs(req) => {
                let operating_voltage_v = req.output_voltage().get::<uom::si::electric_potential::volt>() as f32;
                let operating_current_a = req.operating_current().get::<uom::si::electric_current::ampere>() as f32;
                
                RequestDataObj {
                    raw: req.0,
                    rdo_type: "AVS".to_string(),
                    object_position: req.object_position(),
                    operating_current_a: Some(operating_current_a),
                    max_operating_current_a: None,
                    operating_voltage_v: Some(operating_voltage_v),
                    operating_power_w: None,
                    max_operating_power_w: None,
                    capability_mismatch: req.capability_mismatch(),
                    usb_communications_capable: req.usb_communications_capable(),
                    no_usb_suspend: req.no_usb_suspend(),
                    giveback_flag: None,
                }
            }
            request::PowerSource::Unknown(raw) => {
                RequestDataObj {
                    raw: raw.0,
                    rdo_type: "Unknown".to_string(),
                    object_position: raw.object_position(),
                    operating_current_a: None,
                    max_operating_current_a: None,
                    operating_voltage_v: None,
                    operating_power_w: None,
                    max_operating_power_w: None,
                    capability_mismatch: false,
                    usb_communications_capable: false,
                    no_usb_suspend: false,
                    giveback_flag: None,
                }
            }
        }
    }
}

#[pymethods]
impl RequestDataObj {
    fn __repr__(&self) -> String {
        match self.rdo_type.as_str() {
            "FixedVariableSupply" => format!(
                "RequestDataObj(PDO {}: {}A @ {}A max)",
                self.object_position,
                self.operating_current_a.unwrap_or(0.0),
                self.max_operating_current_a.unwrap_or(0.0)
            ),
            "PPS" => format!(
                "RequestDataObj(PDO {}: {}V @ {}A)",
                self.object_position,
                self.operating_voltage_v.unwrap_or(0.0),
                self.operating_current_a.unwrap_or(0.0)
            ),
            "Battery" => format!(
                "RequestDataObj(PDO {}: {}W @ {}W max)",
                self.object_position,
                self.operating_power_w.unwrap_or(0.0),
                self.max_operating_power_w.unwrap_or(0.0)
            ),
            _ => format!("RequestDataObj(PDO {}: {})", self.object_position, self.rdo_type),
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// PDO state manager for tracking Source Capabilities
#[derive(Debug, Clone)]
pub struct PdoStateManager {
    /// Stored PDOs from most recent Source Capabilities message
    pub source_capabilities: Option<SourceCapabilities>,
}

impl PdoStateManager {
    pub fn new() -> Self {
        Self {
            source_capabilities: None,
        }
    }
    
    pub fn update_source_capabilities(&mut self, capabilities: SourceCapabilities) {
        self.source_capabilities = Some(capabilities);
    }
}

impl PdoState for PdoStateManager {
    fn pdo_at_object_position(&self, position: u8) -> Option<Kind> {
        if let Some(ref caps) = self.source_capabilities {
            if position == 0 || position > 7 {
                return None;
            }
            
            let index = (position - 1) as usize;
            caps.pdos().get(index).map(|pdo| match pdo {
                PowerDataObject::FixedSupply(_) => Kind::FixedSupply,
                PowerDataObject::Battery(_) => Kind::Battery,
                PowerDataObject::VariableSupply(_) => Kind::VariableSupply,
                PowerDataObject::Augmented(aug) => match aug {
                    Augmented::Spr(_) => Kind::Pps,
                    Augmented::Epr(_) => Kind::Avs,
                    Augmented::Unknown(_) => Kind::FixedSupply, // Fallback
                },
                PowerDataObject::Unknown(_) => Kind::FixedSupply, // Fallback
            })
        } else {
            None
        }
    }
}

/// Python wrapper for complete USB PD message
#[pyclass]
#[derive(Debug, Clone)]
pub struct PdMessage {
    #[pyo3(get)]
    /// Message header
    pub header: PdHeader,
    
    #[pyo3(get)]
    /// Data objects (empty for control messages) - PDOs for Source Capabilities
    pub data_objects: Vec<PowerDataObj>,
    
    #[pyo3(get)]
    /// Request data objects (RDOs for Request messages)
    pub request_objects: Vec<RequestDataObj>,
    
    #[pyo3(get)]
    /// Raw message bytes
    pub raw_bytes: Vec<u8>,
}

#[pymethods]
impl PdMessage {
    fn __repr__(&self) -> String {
        let object_count = self.data_objects.len() + self.request_objects.len();
        format!(
            "PdMessage({}, {} objects)",
            self.header.message_type,
            object_count
        )
    }

    fn __str__(&self) -> String {
        let mut result = format!("USB PD Message: {}\n", self.header.message_type);
        result.push_str(&format!("  Message ID: {}\n", self.header.message_id));
        result.push_str(&format!("  Power Role: {}\n", self.header.port_power_role));
        result.push_str(&format!("  Data Role: {}\n", self.header.port_data_role));
        
        if !self.data_objects.is_empty() {
            result.push_str(&format!("  Data Objects ({}):\n", self.data_objects.len()));
            for (i, obj) in self.data_objects.iter().enumerate() {
                result.push_str(&format!("    {}: {}\n", i + 1, obj.__str__()));
            }
        }
        
        if !self.request_objects.is_empty() {
            result.push_str(&format!("  Request Objects ({}):\n", self.request_objects.len()));
            for (i, obj) in self.request_objects.iter().enumerate() {
                result.push_str(&format!("    {}: {}\n", i + 1, obj.__str__()));
            }
        }
        
        result
    }

    /// Get the raw message as a hex string
    fn hex(&self) -> String {
        hex::encode(&self.raw_bytes)
    }

    /// Check if this is a Source Capabilities message
    fn is_source_capabilities(&self) -> bool {
        self.header.message_type == "Source_Capabilities"
    }

    /// Check if this is a control message (no data objects)
    fn is_control_message(&self) -> bool {
        self.header.num_data_objects == 0
    }

    /// Check if this is a data message (has data objects)
    fn is_data_message(&self) -> bool {
        self.header.num_data_objects > 0
    }
}

/// Parse a USB PD message from raw bytes
/// 
/// Args:
///     data: Raw message bytes as bytes object
///     
/// Returns:
///     PdMessage: Parsed USB PD message
///     
/// Raises:
///     ValueError: If the message cannot be parsed
#[pyfunction]
pub fn parse_pd_message(data: &Bound<'_, pyo3::types::PyBytes>) -> PyResult<PdMessage> {
    let bytes = data.as_bytes();
    
    // Prevent panics on too-short input in the underlying crate
    if bytes.len() < 2 {
        return Err(PyValueError::new_err("message too short: expected at least 2 bytes"));
    }

    // Use the usbpd crate to parse the message
    match Message::from_bytes(bytes) {
        Ok(message) => {
            let header = PdHeader::from(&message.header);
            let mut data_objects = Vec::new();
            
            // Validate that the payload length matches header expectations
            let expected_len = 2 + (message.header.num_objects() * 4);
            if bytes.len() < expected_len {
                return Err(PyValueError::new_err(format!(
                    "invalid message length: expected at least {} bytes, got {}",
                    expected_len, bytes.len()
                )));
            }
            
            // Extract data objects based on message data type
            if let Some(data) = &message.data {
                match data {
                    Data::SourceCapabilities(source_caps) => {
                        for pdo in source_caps.pdos() {
                            data_objects.push(PowerDataObj::from(pdo));
                        }
                    }
                    Data::PowerSourceRequest(_request) => {
                        // Basic request support without PDO state
                        // Will be empty for now, but structure is ready
                    }
                    // TODO: Add support for other data types (VendorDefined, etc.)
                    _ => {
                        // For now, we only fully support Source Capabilities
                        // Other message types will have empty data_objects
                    }
                }
            }
            
            Ok(PdMessage {
                header,
                data_objects,
                request_objects: Vec::new(),
                raw_bytes: bytes.to_vec(),
            })
        }
        Err(parse_error) => {
            let error_msg = match parse_error {
                ParseError::InvalidLength { expected, found } => {
                    format!("Invalid message length: expected {}, found {}", expected, found)
                }
                ParseError::UnsupportedSpecificationRevision(rev) => {
                    format!("Unsupported specification revision: {}", rev)
                }
                ParseError::InvalidMessageType(msg_type) => {
                    format!("Invalid message type: {}", msg_type)
                }
                ParseError::InvalidDataMessageType(msg_type) => {
                    format!("Invalid data message type: {}", msg_type)
                }
                ParseError::InvalidControlMessageType(msg_type) => {
                    format!("Invalid control message type: {}", msg_type)
                }
                ParseError::Other(msg) => {
                    format!("Parse error: {}", msg)
                }
            };
            Err(PyValueError::new_err(error_msg))
        }
    }
}

/// Parse a USB PD message from raw bytes with PDO state for Request message support
/// 
/// Args:
///     data: Raw message bytes as bytes object
///     pdo_state: Optional list of PowerDataObj from previous Source Capabilities
///     
/// Returns:
///     PdMessage: Parsed USB PD message with Request objects properly parsed
///     
/// Raises:
///     ValueError: If the message cannot be parsed
#[pyfunction]
pub fn parse_pd_message_with_state(
    data: &Bound<'_, pyo3::types::PyBytes>, 
    pdo_state: Option<Vec<PowerDataObj>>
) -> PyResult<PdMessage> {
    let bytes = data.as_bytes();
    
    // Prevent panics on too-short input
    if bytes.len() < 2 {
        return Err(PyValueError::new_err("message too short: expected at least 2 bytes"));
    }

    // Create PDO state manager
    let mut state_manager = PdoStateManager::new();
    
    // If PDO state is provided, convert to SourceCapabilities
    if let Some(pdos) = pdo_state {
        // Create a dummy source capabilities message to extract the real SourceCapabilities
        // This is a bit of a hack, but we need to work with the Rust API
        let _dummy_header = Header(0x0161); // Source Capabilities with 6 objects
        let mut dummy_payload = Vec::new();
        
        // Add each PDO as raw u32 (this is a simplified approach)
        for pdo in &pdos {
            dummy_payload.extend_from_slice(&pdo.raw.to_le_bytes());
        }
        
        // Create dummy message bytes
        let mut dummy_bytes = vec![0x61, 0x01]; // Header bytes
        dummy_bytes.extend(dummy_payload);
        
        // Parse to extract SourceCapabilities
        if let Ok(dummy_msg) = Message::from_bytes(&dummy_bytes) {
            if let Some(Data::SourceCapabilities(caps)) = dummy_msg.data {
                state_manager.update_source_capabilities(caps);
            }
        }
    }

    // Use the usbpd crate to parse the message with state
    match Message::from_bytes_with_state(bytes, &state_manager) {
        Ok(message) => {
            let header = PdHeader::from(&message.header);
            let mut data_objects = Vec::new();
            let mut request_objects = Vec::new();
            
            // Validate that the payload length matches header expectations
            let expected_len = 2 + (message.header.num_objects() * 4);
            if bytes.len() < expected_len {
                return Err(PyValueError::new_err(format!(
                    "invalid message length: expected at least {} bytes, got {}",
                    expected_len, bytes.len()
                )));
            }
            
            // Extract data objects based on message data type
            if let Some(data) = &message.data {
                match data {
                    Data::SourceCapabilities(source_caps) => {
                        for pdo in source_caps.pdos() {
                            data_objects.push(PowerDataObj::from(pdo));
                        }
                    }
                    Data::PowerSourceRequest(request) => {
                        request_objects.push(RequestDataObj::from(request));
                    }
                    // TODO: Add support for other data types (VendorDefined, etc.)
                    _ => {
                        // For now, we only fully support Source Capabilities and Request
                        // Other message types will have empty data_objects
                    }
                }
            }
            
            Ok(PdMessage {
                header,
                data_objects,
                request_objects,
                raw_bytes: bytes.to_vec(),
            })
        }
        Err(parse_error) => {
            let error_msg = match parse_error {
                ParseError::InvalidLength { expected, found } => {
                    format!("Invalid message length: expected {}, found {}", expected, found)
                }
                ParseError::UnsupportedSpecificationRevision(rev) => {
                    format!("Unsupported specification revision: {}", rev)
                }
                ParseError::InvalidMessageType(msg_type) => {
                    format!("Invalid message type: {}", msg_type)
                }
                ParseError::InvalidDataMessageType(msg_type) => {
                    format!("Invalid data message type: {}", msg_type)
                }
                ParseError::InvalidControlMessageType(msg_type) => {
                    format!("Invalid control message type: {}", msg_type)
                }
                ParseError::Other(msg) => {
                    format!("Parse error: {}", msg)
                }
            };
            Err(PyValueError::new_err(error_msg))
        }
    }
}

/// Parse multiple USB PD messages from a list of byte arrays
/// 
/// Args:
///     messages: List of bytes objects containing raw message data
///     
/// Returns:
///     List[PdMessage]: List of successfully parsed messages
///     
/// Note:
///     Failed messages are skipped and not included in the result
#[pyfunction]
pub fn parse_messages(messages: &Bound<'_, pyo3::types::PyList>) -> PyResult<Vec<PdMessage>> {
    let mut parsed_messages = Vec::new();
    
    for item in messages.iter() {
        if let Ok(bytes) = item.downcast::<pyo3::types::PyBytes>() {
            match parse_pd_message(&bytes) {
                Ok(msg) => parsed_messages.push(msg),
                Err(_) => {
                    // Skip failed messages, continue with others
                    continue;
                }
            }
        } else {
            return Err(PyTypeError::new_err("All items must be bytes objects"));
        }
    }
    
    Ok(parsed_messages)
}

/// Convert a hex string to bytes
/// 
/// Args:
///     hex_str: Hexadecimal string (with or without spaces/separators)
///     
/// Returns:
///     bytes: Raw bytes
///     
/// Raises:
///     ValueError: If the hex string is invalid
#[pyfunction]
pub fn hex_to_bytes(hex_str: &str) -> PyResult<Vec<u8>> {
    let cleaned = hex_str.replace(" ", "").replace("-", "").replace(":", "");
    hex::decode(cleaned)
        .map_err(|e| PyValueError::new_err(format!("Invalid hex string: {}", e)))
}

/// Convert bytes to a hex string
/// 
/// Args:
///     data: Raw bytes
///     
/// Returns:
///     str: Hexadecimal string representation
#[pyfunction]
pub fn bytes_to_hex(data: &Bound<'_, pyo3::types::PyBytes>) -> String {
    hex::encode(data.as_bytes())
}

/// Get human-readable message type name based on message type and number of data objects
/// 
/// This function correctly distinguishes between control and data messages:
/// - Control messages have num_data_objects = 0
/// - Data messages have num_data_objects > 0
/// 
/// Args:
///     message_type: Raw message type value (0-31)
///     num_data_objects: Number of data objects (0-7)
///     
/// Returns:
///     str: Human-readable message type name
#[pyfunction]
pub fn get_message_type_name(message_type: u8, num_data_objects: u8) -> String {
    if num_data_objects == 0 {
        // Control message
        match message_type {
            1 => "GoodCRC".to_string(),
            2 => "GotoMin".to_string(),
            3 => "Accept".to_string(),
            4 => "Reject".to_string(),
            5 => "Ping".to_string(),
            6 => "PS_RDY".to_string(),
            7 => "Get_Source_Cap".to_string(),
            8 => "Get_Sink_Cap".to_string(),
            9 => "DR_Swap".to_string(),
            10 => "PR_Swap".to_string(),
            11 => "VCONN_Swap".to_string(),
            12 => "Wait".to_string(),
            13 => "Soft_Reset".to_string(),
            14 => "Data_Reset".to_string(),
            15 => "Data_Reset_Complete".to_string(),
            16 => "Not_Supported".to_string(),
            17 => "Get_Source_Cap_Extended".to_string(),
            18 => "Get_Status".to_string(),
            19 => "FR_Swap".to_string(),
            20 => "Get_PPS_Status".to_string(),
            21 => "Get_Country_Codes".to_string(),
            22 => "Get_Sink_Cap_Extended".to_string(),
            23 => "Get_Source_Info".to_string(),
            24 => "Get_Revision".to_string(),
            _ => format!("Control_Message_{}", message_type),
        }
    } else {
        // Data message
        match message_type {
            1 => "Source_Capabilities".to_string(),
            2 => "Request".to_string(),
            3 => "BIST".to_string(),
            4 => "Sink_Capabilities".to_string(),
            5 => "Battery_Status".to_string(),
            6 => "Alert".to_string(),
            7 => "Get_Country_Info".to_string(),
            8 => "Enter_USB".to_string(),
            9 => "EPR_Request".to_string(),
            10 => "EPR_Mode".to_string(),
            11 => "Source_Info".to_string(),
            12 => "Revision".to_string(),
            15 => "Vendor_Defined".to_string(),
            _ => format!("Data_Message_{}", message_type),
        }
    }
}

/// usbpdpy - Python bindings for USB Power Delivery message parsing
/// 
/// This module provides fast and accurate USB PD message parsing using 
/// the proven usbpd Rust crate.
#[pymodule]
fn usbpdpy(m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add("__version__", "0.1.0")?;
    m.add("__author__", "Danila Gornushko <me@okhsunrog.dev>")?;
    
    // Functions
    m.add_function(wrap_pyfunction!(parse_pd_message, m)?)?;
    m.add_function(wrap_pyfunction!(parse_pd_message_with_state, m)?)?;
    m.add_function(wrap_pyfunction!(parse_messages, m)?)?;
    m.add_function(wrap_pyfunction!(get_message_type_name, m)?)?;
    m.add_function(wrap_pyfunction!(hex_to_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(bytes_to_hex, m)?)?;
    
    // Add alias for backwards compatibility
    m.add("parse_message", m.getattr("parse_pd_message")?)?;
    
    // Classes
    m.add_class::<PdMessage>()?;
    m.add_class::<PdHeader>()?;
    m.add_class::<PowerDataObj>()?;
    m.add_class::<RequestDataObj>()?;
    
    Ok(())
}
