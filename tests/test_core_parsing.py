"""
Core USB PD message parsing tests
"""
import pytest
import usbpdpy

try:
    from pyo3_runtime import PanicException
except ImportError:
    class PanicException(Exception):
        pass


class TestCoreParsing:
    """Test core USB PD message parsing functionality"""
    
    def test_source_capabilities_parsing(self):
        """Test parsing Source Capabilities message"""
        # 26-byte Source Capabilities message with 6 PDOs
        source_caps_hex = "a1612c9101082cd102002cc103002cb10400454106003c21dcc0"
        source_caps_bytes = bytes.fromhex(source_caps_hex)
        
        message = usbpdpy.parse_message(source_caps_bytes)
        
        # Verify header
        assert message.header.message_type_raw == 1
        assert message.header.num_data_objects == 6
        assert message.header.port_power_role == "Source"
        assert message.header.port_data_role == "Dfp"
        
        # Verify message classification
        assert message.is_source_capabilities()
        assert message.is_data_message()
        assert not message.is_control_message()
        
        # Verify data objects
        assert len(message.data_objects) == 6
        
        # Check first PDO (should be 5V fixed supply)
        pdo1 = message.data_objects[0]
        assert pdo1.voltage_v == 5.0
        assert pdo1.max_current_a > 0
        
    def test_goodcrc_parsing(self):
        """Test parsing real GoodCRC message"""
        # Real GoodCRC: message_type=1, num_objects=0
        goodcrc_bytes = bytes([0x41, 0x01])
        
        message = usbpdpy.parse_message(goodcrc_bytes)
        
        # Verify header
        assert message.header.message_type_raw == 1
        assert message.header.num_data_objects == 0
        
        # Verify message classification
        assert not message.is_source_capabilities()
        assert message.is_control_message()
        assert not message.is_data_message()
        
        # Verify no data objects
        assert len(message.data_objects) == 0
        
    def test_message_type_differentiation(self):
        """Test that Source Capabilities and GoodCRC are correctly differentiated"""
        # Source Capabilities (type=1, objects=6)
        source_caps_bytes = bytes.fromhex("a1612c9101082cd102002cc103002cb10400454106003c21dcc0")
        source_msg = usbpdpy.parse_message(source_caps_bytes)
        
        # GoodCRC (type=1, objects=0)
        goodcrc_bytes = bytes([0x41, 0x01])
        goodcrc_msg = usbpdpy.parse_message(goodcrc_bytes)
        
        # Both have message_type=1 but different object counts
        assert source_msg.header.message_type_raw == goodcrc_msg.header.message_type_raw == 1
        assert source_msg.header.num_data_objects == 6
        assert goodcrc_msg.header.num_data_objects == 0
        
        # But they should be classified differently
        source_name = usbpdpy.get_message_type_name(
            source_msg.header.message_type_raw,
            source_msg.header.num_data_objects
        )
        goodcrc_name = usbpdpy.get_message_type_name(
            goodcrc_msg.header.message_type_raw,
            goodcrc_msg.header.num_data_objects
        )
        
        assert source_name == "Source_Capabilities"
        assert goodcrc_name == "GoodCRC"
        
    def test_short_message_error(self):
        """Test error handling for messages that are too short"""
        # Any exception is fine - the important thing is it doesn't crash
        with pytest.raises(Exception):
            usbpdpy.parse_message(bytes([0x41]))  # Only 1 byte
            
    def test_empty_message_error(self):
        """Test error handling for empty messages"""
        # Any exception is fine - the important thing is it doesn't crash
        with pytest.raises(Exception):
            usbpdpy.parse_message(bytes())


class TestPowerDataObjects:
    """Test Power Data Object parsing"""
    
    def test_fixed_supply_pdo(self):
        """Test Fixed Supply PDO parsing"""
        # Create a message with fixed supply PDOs
        source_caps_bytes = bytes.fromhex("a1612c9101082cd102002cc103002cb10400454106003c21dcc0")
        message = usbpdpy.parse_message(source_caps_bytes)
        
        # Should have multiple PDOs
        assert len(message.data_objects) > 0
        
        # Check first PDO properties
        pdo = message.data_objects[0]
        assert hasattr(pdo, 'voltage_v')
        assert hasattr(pdo, 'max_current_a')
        assert pdo.voltage_v > 0
        assert pdo.max_current_a > 0
        
    def test_pdo_power_calculation(self):
        """Test that PDO power calculations are reasonable"""
        source_caps_bytes = bytes.fromhex("a1612c9101082cd102002cc103002cb10400454106003c21dcc0")
        message = usbpdpy.parse_message(source_caps_bytes)
        
        # Check that PDOs have reasonable power values
        for pdo in message.data_objects:
            if hasattr(pdo, 'max_power_w') and pdo.max_power_w:
                assert 0 < pdo.max_power_w <= 240  # USB PD max is 240W
                
            if hasattr(pdo, 'voltage_v') and hasattr(pdo, 'max_current_a'):
                if pdo.voltage_v and pdo.max_current_a:
                    power_w = pdo.voltage_v * pdo.max_current_a
                    assert 0 < power_w <= 240


class TestUtilityFunctions:
    """Test utility functions"""
    
    def test_hex_conversion(self):
        """Test hex string conversion functions"""
        test_hex = "a1612c91"
        
        # Convert hex to bytes
        result_bytes = usbpdpy.hex_to_bytes(test_hex)
        assert isinstance(result_bytes, bytes)
        assert len(result_bytes) == 4
        
        # Convert back to hex
        hex_result = usbpdpy.bytes_to_hex(result_bytes)
        assert hex_result.lower() == test_hex.lower()
        
    def test_hex_conversion_edge_cases(self):
        """Test hex conversion edge cases"""
        # Empty string
        empty_result = usbpdpy.hex_to_bytes("")
        assert empty_result == b''
        
        # Single byte
        single_result = usbpdpy.hex_to_bytes("ff")
        assert single_result == b'\xff'
