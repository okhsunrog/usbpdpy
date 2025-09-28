# usbpdpy

Python bindings for USB Power Delivery message parsing using the [`usbpd`](https://crates.io/crates/usbpd) Rust crate.

## Features

- Parse USB PD messages with full specification compliance
- Support for Source Capabilities and Request message parsing
- All PDO types: Fixed Supply, Battery, Variable Supply, PPS, EPR
- Complete RDO (Request Data Object) parsing with PDO state management
- Message header parsing with proper control/data message classification
- Python type hints and error handling

## Installation

```bash
pip install usbpdpy
```

## Quick Start

### Parse Source Capabilities

```python
import usbpdpy

# Parse a Source Capabilities message
source_caps_hex = "a1612c9101082cd102002cc103002cb10400454106003c21dcc0"
source_caps_bytes = bytes.fromhex(source_caps_hex)
message = usbpdpy.parse_pd_message(source_caps_bytes)

print(f"Message: {message.header.message_type}")
print(f"Power role: {message.header.port_power_role}")
print(f"Available PDOs: {len(message.data_objects)}")

for i, pdo in enumerate(message.data_objects):
    print(f"  PDO {i+1}: {pdo}")
    # Output: PDO 1: PowerDataObj(FixedSupply: 5V @ 3A = 15W)
    #         PDO 2: PowerDataObj(FixedSupply: 9V @ 3A = 27W)
    #         ...
```

### Parse Request Messages

```python
# First, parse Source Capabilities to get PDO state
source_msg = usbpdpy.parse_pd_message(source_caps_bytes)

# Parse Request message with PDO context
request_hex = "8210dc700323"
request_bytes = bytes.fromhex(request_hex)
request_msg = usbpdpy.parse_pd_message_with_state(request_bytes, source_msg.data_objects)

print(f"Request type: {request_msg.header.message_type}")
for rdo in request_msg.request_objects:
    print(f"  Requesting PDO #{rdo.object_position}: {rdo.rdo_type}")
    # Output: Requesting PDO #2: FixedVariableSupply
```

## API Reference

### Core Functions

- `parse_pd_message(data: bytes) -> PdMessage` - Parse a USB PD message
- `parse_pd_message_with_state(data: bytes, pdo_state: List[PowerDataObj]) -> PdMessage` - Parse with PDO context for Request messages
- `parse_messages(messages: List[bytes]) -> List[PdMessage]` - Parse multiple messages
- `get_message_type_name(msg_type: int, num_objects: int) -> str` - Get human-readable message type
- `hex_to_bytes(hex_str: str) -> List[int]` - Convert hex string to byte list
- `bytes_to_hex(data: bytes) -> str` - Convert bytes to hex string

### Message Structure

**PdMessage**
- `header: PdHeader` - Message header information
- `data_objects: List[PowerDataObj]` - Power Data Objects (PDOs) for Source/Sink Capabilities
- `request_objects: List[RequestDataObj]` - Request Data Objects (RDOs) for Request messages
- `raw_bytes: bytes` - Original message bytes
- `is_control_message() -> bool` - Check if control message
- `is_data_message() -> bool` - Check if data message  
- `is_source_capabilities() -> bool` - Check if Source Capabilities
- `hex() -> str` - Get message as hex string

**PdHeader**  
- `message_type: str` - Human-readable type (e.g., "Source_Capabilities", "GoodCRC")
- `message_type_raw: int` - Raw type code (0–31)
- `port_data_role: str` - "Ufp" or "Dfp"
- `port_power_role: str` - "Sink" or "Source"
- `message_id: int` - Message ID (0–7)
- `num_data_objects: int` - Number of 32-bit data objects (0–7)
- `spec_revision: int` - PD spec rev (0=R1.0, 1=R2.0, 2=R3.0)
- `extended: bool` - Extended message flag

**PowerDataObj**
- `pdo_type: str` - "FixedSupply", "Battery", "VariableSupply", "PPS", "EPR_AVS", "Unknown"
- `raw: int` - Raw 32-bit PDO value
- `voltage_v: Optional[float]` - Voltage in volts (fixed supply)
- `max_current_a: Optional[float]` - Maximum current in amperes
- `max_power_w: Optional[float]` - Maximum power in watts (calculated or battery)
- `min_voltage_v: Optional[float]` - Minimum voltage in volts (variable/PPS)
- `max_voltage_v: Optional[float]` - Maximum voltage in volts (variable/PPS)
- `dual_role_power: Optional[bool]` - Dual role power capability
- `usb_communications_capable: Optional[bool]` - USB communications capability
- `unconstrained_power: Optional[bool]` - Unconstrained power flag

**RequestDataObj**
- `rdo_type: str` - "FixedVariableSupply", "Battery", "PPS", "AVS", "Unknown"
- `raw: int` - Raw 32-bit RDO value
- `object_position: int` - PDO position being requested (1-7)
- `operating_current_a: Optional[float]` - Operating current in amperes
- `max_operating_current_a: Optional[float]` - Maximum operating current in amperes
- `operating_voltage_v: Optional[float]` - Operating voltage in volts (PPS/AVS)
- `operating_power_w: Optional[float]` - Operating power in watts (battery)
- `max_operating_power_w: Optional[float]` - Maximum operating power in watts (battery)
- `capability_mismatch: bool` - Capability mismatch flag
- `usb_communications_capable: bool` - USB communications capability
- `no_usb_suspend: bool` - No USB suspend flag
- `giveback_flag: Optional[bool]` - GiveBack flag (fixed/variable supply)

## Message Types

The library correctly distinguishes between control and data messages:

- **Control Messages** (`num_data_objects = 0`): GoodCRC, Accept, Reject, PS_RDY, etc.
- **Data Messages** (`num_data_objects > 0`): Source_Capabilities, Request, Sink_Capabilities, etc.

### Supported Data Messages

- **Source_Capabilities**: Parsed into `data_objects` (PDOs)
- **Request**: Parsed into `request_objects` (RDOs) when PDO state is provided
- **Sink_Capabilities**: Header parsed, data objects pending
- **Other data messages**: Header parsed, data objects pending

## Real-World Usage

This library has been tested with real USB PD captures from KM003C hardware, including complete negotiation sequences:

```python
# Parse a complete USB PD negotiation sequence
messages = [
    "a1612c9101082cd102002cc103002cb10400454106003c21dcc0",  # Source_Capabilities
    "8210dc700323",  # Request (9V)
    "a305",          # Accept
    "a607",          # PS_RDY
]

source_msg = usbpdpy.parse_pd_message(bytes.fromhex(messages[0]))
request_msg = usbpdpy.parse_pd_message_with_state(
    bytes.fromhex(messages[1]), 
    source_msg.data_objects
)

print(f"Sink requested PDO #{request_msg.request_objects[0].object_position}")
# Output: Sink requested PDO #2 (9V from the source capabilities)
```

## Development

```bash
# Install development dependencies
uv sync --dev

# Build the extension
uv run maturin develop

# Run tests
uv run pytest

# Lint and format
uv run ruff check .
uv run ruff format .
```

## Requirements

- Python 3.8+
- No runtime dependencies

## License

MIT
