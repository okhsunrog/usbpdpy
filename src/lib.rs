use pyo3::prelude::*;
use pyo3::types::{PyBytesMethods, PyListMethods};

/// Python wrapper for USB PD Header
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyPdHeader {
    #[pyo3(get)]
    pub message_type: u8,
    #[pyo3(get)]
    pub port_data_role: String,
    #[pyo3(get)]
    pub specification_revision: u8,
    #[pyo3(get)]
    pub port_power_role: String,
    #[pyo3(get)]
    pub message_id: u8,
    #[pyo3(get)]
    pub number_of_data_objects: u8,
    #[pyo3(get)]
    pub extended: bool,
}

#[pymethods]
impl PyPdHeader {
    fn __repr__(&self) -> String {
        format!(
            "PyPdHeader(type={}, data_role={}, power_role={}, msg_id={}, num_objects={})",
            self.message_type,
            self.port_data_role,
            self.port_power_role,
            self.message_id,
            self.number_of_data_objects
        )
    }

    fn to_dict(&self, py: Python) -> PyResult<Py<pyo3::PyAny>> {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("message_type", self.message_type)?;
        dict.set_item("port_data_role", &self.port_data_role)?;
        dict.set_item("specification_revision", self.specification_revision)?;
        dict.set_item("port_power_role", &self.port_power_role)?;
        dict.set_item("message_id", self.message_id)?;
        dict.set_item("number_of_data_objects", self.number_of_data_objects)?;
        dict.set_item("extended", self.extended)?;
        Ok(dict.into())
    }
}

/// Python wrapper for USB PD Data Object
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyPdDataObject {
    #[pyo3(get)]
    pub raw: u32,
    #[pyo3(get)]
    pub object_type: String,
    #[pyo3(get)]
    pub parsed_data: Option<String>, // JSON string of parsed data
}

#[pymethods]
impl PyPdDataObject {
    fn __repr__(&self) -> String {
        format!("PyPdDataObject(type={}, raw=0x{:08x})", self.object_type, self.raw)
    }

    fn to_dict(&self, py: Python) -> PyResult<Py<pyo3::PyAny>> {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("raw", self.raw)?;
        dict.set_item("object_type", &self.object_type)?;
        dict.set_item("parsed_data", &self.parsed_data)?;
        Ok(dict.into())
    }
}

/// Python wrapper for complete USB PD Message
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyPdMessage {
    #[pyo3(get)]
    pub header: PyPdHeader,
    #[pyo3(get)]
    pub data_objects: Vec<PyPdDataObject>,
    #[pyo3(get)]
    pub raw_bytes: Vec<u8>,
}

#[pymethods]
impl PyPdMessage {
    fn __repr__(&self) -> String {
        format!(
            "PyPdMessage(type={}, objects={}, raw_len={})",
            self.header.message_type,
            self.data_objects.len(),
            self.raw_bytes.len()
        )
    }

    fn to_dict(&self, py: Python) -> PyResult<Py<pyo3::PyAny>> {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("header", self.header.to_dict(py)?)?;
        
        let objects = pyo3::types::PyList::empty(py);
        for obj in &self.data_objects {
            objects.append(obj.to_dict(py)?)?;
        }
        dict.set_item("data_objects", &objects)?;
        dict.set_item("raw_bytes", &self.raw_bytes)?;
        Ok(dict.into())
    }

    fn get_hex(&self) -> String {
        hex::encode(&self.raw_bytes)
    }
}

/// Parse a USB PD message from raw bytes (simplified version without usbpd crate for now)
#[pyfunction]
pub fn parse_pd_message(data: &Bound<'_, pyo3::types::PyBytes>) -> PyResult<PyPdMessage> {
    let bytes = data.as_bytes();
    
    if bytes.len() < 2 {
        return Err(pyo3::exceptions::PyValueError::new_err("Message too short"));
    }
    
    // Simple parsing - this is a placeholder until we fix the usbpd crate integration
    let header_word = u16::from_le_bytes([bytes[0], bytes[1]]);
    
    let header = PyPdHeader {
        message_type: (header_word & 0x1F) as u8,
        port_data_role: if (header_word & 0x20) != 0 { "DFP".to_string() } else { "UFP".to_string() },
        specification_revision: ((header_word >> 6) & 0x3) as u8,
        port_power_role: if (header_word & 0x100) != 0 { "Source".to_string() } else { "Sink".to_string() },
        message_id: ((header_word >> 9) & 0x7) as u8,
        number_of_data_objects: ((header_word >> 12) & 0x7) as u8,
        extended: (header_word & 0x8000) != 0,
    };

    let data_objects = Vec::new(); // Simplified - no data object parsing yet
    
    Ok(PyPdMessage {
        header,
        data_objects,
        raw_bytes: bytes.to_vec(),
    })
}

/// Parse multiple USB PD messages from a list of byte arrays
#[pyfunction]
pub fn parse_pd_messages(messages: &Bound<'_, pyo3::types::PyList>) -> PyResult<Vec<PyPdMessage>> {
    let mut parsed_messages = Vec::new();
    
    for item in messages.iter() {
        if let Ok(bytes) = item.downcast::<pyo3::types::PyBytes>() {
            match parse_pd_message(&bytes) {
                Ok(msg) => parsed_messages.push(msg),
                Err(e) => {
                    // Log error but continue with other messages
                    eprintln!("Warning: Failed to parse message: {}", e);
                }
            }
        }
    }
    
    Ok(parsed_messages)
}

/// Get message type name from message type number
#[pyfunction]
pub fn get_message_type_name(msg_type: u8) -> String {
    match msg_type {
        0 => "Reserved".to_string(),
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
        _ => format!("Unknown({})", msg_type),
    }
}

/// Utility function to convert hex string to bytes
#[pyfunction]
pub fn hex_to_bytes(hex_str: &str) -> PyResult<Vec<u8>> {
    hex::decode(hex_str.trim())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid hex string: {}", e)))
}

/// Utility function to convert bytes to hex string
#[pyfunction]
pub fn bytes_to_hex(data: &Bound<'_, pyo3::types::PyBytes>) -> String {
    hex::encode(data.as_bytes())
}

/// A Python module implemented in Rust for USB PD message parsing
#[pymodule]
fn usbpdpy(m: &Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_pd_message, m)?)?;
    m.add_function(wrap_pyfunction!(parse_pd_messages, m)?)?;
    m.add_function(wrap_pyfunction!(get_message_type_name, m)?)?;
    m.add_function(wrap_pyfunction!(hex_to_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(bytes_to_hex, m)?)?;
    
    m.add_class::<PyPdMessage>()?;
    m.add_class::<PyPdHeader>()?;
    m.add_class::<PyPdDataObject>()?;
    
    Ok(())
}