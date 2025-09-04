# usbpdpy

Python bindings for the [`usbpd`](https://crates.io/crates/usbpd) Rust crate.

## Installation

```bash
pip install usbpdpy
```

## Usage

```python
import usbpdpy

# Parse message from bytes
message = usbpdpy.parse_pd_message(b'\x61\x11')
print(f"Type: {message.header.message_type}")
print(f"Name: {usbpdpy.get_message_type_name(message.header.message_type)}")

# Convert hex to bytes
data = usbpdpy.hex_to_bytes("1161")
message = usbpdpy.parse_pd_message(data)
```

## API

### Functions

- `parse_pd_message(data: bytes) -> PyPdMessage`
- `parse_pd_messages(messages: List[bytes]) -> List[PyPdMessage]`
- `get_message_type_name(msg_type: int) -> str`
- `hex_to_bytes(hex_str: str) -> bytes`
- `bytes_to_hex(data: bytes) -> str`

### Classes

**PyPdMessage**
- `header: PyPdHeader`
- `data_objects: List[PyPdDataObject]` 
- `raw_bytes: List[int]`
- `get_hex() -> str`

**PyPdHeader**
- `message_type: int`
- `port_data_role: str` ("UFP"/"DFP")
- `port_power_role: str` ("Sink"/"Source")
- `message_id: int`
- `number_of_data_objects: int`
- `extended: bool`

**PyPdDataObject**
- `raw: int`
- `object_type: str`
- `parsed_data: Optional[str]`

## Requirements

- Python 3.8+
- No runtime dependencies

## License

MIT