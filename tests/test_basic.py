"""
Basic tests for usbpdpy package
"""

import pytest
import usbpdpy


def test_hex_to_bytes():
    """Test hex string to bytes conversion"""
    # Test simple hex string
    result = usbpdpy.hex_to_bytes("1161")
    assert result == b'\x11\x61'
    
    # Test with spaces (remove spaces first)
    result = usbpdpy.hex_to_bytes("1161")
    assert result == b'\x11\x61'
    
    # Test empty string
    result = usbpdpy.hex_to_bytes("")
    assert result == b''


def test_bytes_to_hex():
    """Test bytes to hex string conversion"""
    result = usbpdpy.bytes_to_hex(b'\x11\x61')
    assert result == "1161"
    
    result = usbpdpy.bytes_to_hex(b'')
    assert result == ""


def test_get_message_type_name():
    """Test message type name lookup"""
    assert usbpdpy.get_message_type_name(1) == "GoodCRC"
    assert usbpdpy.get_message_type_name(3) == "Accept"
    assert usbpdpy.get_message_type_name(7) == "Get_Source_Cap"
    assert "Unknown" in usbpdpy.get_message_type_name(255)


def test_parse_pd_message():
    """Test USB PD message parsing"""
    # Test GoodCRC message
    msg_bytes = usbpdpy.hex_to_bytes("1161")
    message = usbpdpy.parse_pd_message(msg_bytes)
    
    assert message.header.message_type == 17  # 0x11 & 0x1F = 17
    assert message.header.port_data_role in ["UFP", "DFP"]
    assert message.header.port_power_role in ["Sink", "Source"]
    assert isinstance(message.header.message_id, int)
    assert isinstance(message.header.number_of_data_objects, int)
    assert isinstance(message.header.extended, bool)
    assert len(message.raw_bytes) == 2
    assert message.get_hex() == "1161"


def test_parse_pd_message_error():
    """Test error handling for invalid messages"""
    with pytest.raises(ValueError):
        usbpdpy.parse_pd_message(b'')  # Too short
    
    with pytest.raises(ValueError):
        usbpdpy.parse_pd_message(b'\x00')  # Too short


def test_parse_pd_messages():
    """Test parsing multiple messages"""
    messages = [b'\x11\x61', b'\x01\x43']
    result = usbpdpy.parse_pd_messages(messages)
    
    assert len(result) == 2
    assert all(hasattr(msg, 'header') for msg in result)
    assert all(hasattr(msg, 'data_objects') for msg in result)


def test_hex_to_bytes_error():
    """Test error handling for invalid hex strings"""
    with pytest.raises(ValueError):
        usbpdpy.hex_to_bytes("invalid_hex")
    
    with pytest.raises(ValueError):
        usbpdpy.hex_to_bytes("11G1")  # Invalid hex character


def test_message_attributes():
    """Test that message objects have expected attributes"""
    msg_bytes = usbpdpy.hex_to_bytes("1161")
    message = usbpdpy.parse_pd_message(msg_bytes)
    
    # Test header attributes
    header = message.header
    assert hasattr(header, 'message_type')
    assert hasattr(header, 'port_data_role')
    assert hasattr(header, 'specification_revision')
    assert hasattr(header, 'port_power_role')
    assert hasattr(header, 'message_id')
    assert hasattr(header, 'number_of_data_objects')
    assert hasattr(header, 'extended')
    
    # Test message attributes
    assert hasattr(message, 'header')
    assert hasattr(message, 'data_objects')
    assert hasattr(message, 'raw_bytes')
    
    # Test methods
    assert callable(message.get_hex)
    assert callable(message.to_dict)


def test_integration():
    """Integration test with multiple operations"""
    # Test complete workflow
    test_messages = ["1161", "0143", "0744"]
    
    parsed_messages = []
    for hex_msg in test_messages:
        msg_bytes = usbpdpy.hex_to_bytes(hex_msg)
        parsed_msg = usbpdpy.parse_pd_message(msg_bytes)
        parsed_messages.append(parsed_msg)
    
    assert len(parsed_messages) == 3
    
    # Verify each message
    for i, msg in enumerate(parsed_messages):
        assert msg.get_hex() == test_messages[i].lower()
        type_name = usbpdpy.get_message_type_name(msg.header.message_type)
        assert isinstance(type_name, str)
        assert len(type_name) > 0


if __name__ == "__main__":
    # Run basic smoke test
    test_hex_to_bytes()
    test_parse_pd_message()
    test_integration()
    print("✅ All basic tests passed!")
