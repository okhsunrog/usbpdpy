"""
Test the Python bindings to the Rust usbpd crate
"""

import pytest
import usbpdpy


class TestBindingsInterface:
    """Test that the Python bindings expose the correct interface"""

    def test_main_parse_function_exists(self):
        """Test that main parsing function exists"""
        assert hasattr(usbpdpy, "parse_message")
        assert callable(usbpdpy.parse_message)

    def test_utility_functions_exist(self):
        """Test that utility functions exist"""
        assert hasattr(usbpdpy, "hex_to_bytes")
        assert hasattr(usbpdpy, "bytes_to_hex")
        assert hasattr(usbpdpy, "get_message_type_name")
        assert hasattr(usbpdpy, "parse_pd_message_with_state")  # New function
        assert hasattr(usbpdpy, "parse_messages")

        assert callable(usbpdpy.hex_to_bytes)
        assert callable(usbpdpy.bytes_to_hex)
        assert callable(usbpdpy.get_message_type_name)
        assert callable(usbpdpy.parse_pd_message_with_state)
        assert callable(usbpdpy.parse_messages)

    def test_parse_function_returns_correct_type(self):
        """Test that parse function returns the expected type"""
        test_bytes = bytes([0x41, 0x01])  # Simple GoodCRC
        result = usbpdpy.parse_message(test_bytes)

        # Should return a message object with all expected fields
        assert hasattr(result, "header")
        assert hasattr(result, "data_objects")
        assert hasattr(result, "request_objects")  # New field
        assert hasattr(result, "raw_bytes")

    def test_header_structure(self):
        """Test that header has expected attributes"""
        test_bytes = bytes([0x41, 0x01])
        message = usbpdpy.parse_message(test_bytes)

        header = message.header

        # Check all expected header fields
        assert hasattr(header, "message_type")
        assert hasattr(header, "port_data_role")
        assert hasattr(header, "spec_revision")
        assert hasattr(header, "port_power_role")
        assert hasattr(header, "message_id")
        assert hasattr(header, "num_data_objects")
        assert hasattr(header, "extended")

        # Values should be reasonable
        assert isinstance(header.message_type_raw, int)
        assert header.message_type_raw >= 0
        assert isinstance(header.num_data_objects, int)
        assert header.num_data_objects >= 0

    def test_message_classification_methods(self):
        """Test that message objects have classification methods"""
        test_bytes = bytes([0x41, 0x01])
        message = usbpdpy.parse_message(test_bytes)

        # Should have classification methods
        assert hasattr(message, "is_control_message")
        assert hasattr(message, "is_data_message")
        assert hasattr(message, "is_source_capabilities")

        assert callable(message.is_control_message)
        assert callable(message.is_data_message)
        assert callable(message.is_source_capabilities)

        # Results should be boolean
        assert isinstance(message.is_control_message(), bool)
        assert isinstance(message.is_data_message(), bool)
        assert isinstance(message.is_source_capabilities(), bool)


class TestRustIntegration:
    """Test that we're actually using the Rust usbpd crate"""

    def test_rust_parsing_behavior(self):
        """Test behavior that indicates we're using the real Rust implementation"""
        # Use a complex message that would be hard to fake
        source_caps_bytes = bytes.fromhex(
            "a1612c9101082cd102002cc103002cb10400454106003c21dcc0"
        )
        message = usbpdpy.parse_message(source_caps_bytes)

        # Should parse header correctly (this requires proper bit manipulation)
        assert message.header.message_type_raw == 1
        assert message.header.num_data_objects == 6
        assert message.header.port_power_role == "Source"

        # Should parse data objects (this requires proper 32-bit word parsing)
        assert len(message.data_objects) == 6

        # First PDO should be 5V (this requires proper PDO parsing logic)
        first_pdo = message.data_objects[0]
        assert first_pdo.voltage_v == 5.0

    def test_error_handling_from_rust(self):
        """Test that Rust errors are properly handled"""
        # Test with invalid data
        with pytest.raises((Exception, ValueError)):
            usbpdpy.parse_message(bytes())  # Empty

        with pytest.raises((Exception, ValueError)):
            usbpdpy.parse_message(bytes([0xFF]))  # Too short


class TestUseCases:
    """Test realistic use cases"""

    def test_power_adapter_analysis(self):
        """Test analyzing a power adapter's capabilities"""
        source_caps_bytes = bytes.fromhex(
            "a1612c9101082cd102002cc103002cb10400454106003c21dcc0"
        )
        message = usbpdpy.parse_message(source_caps_bytes)

        # Should identify as power source
        assert message.header.port_power_role == "Source"
        assert message.is_source_capabilities()

        # Should have multiple power options
        assert len(message.data_objects) > 1

        # Should have reasonable power levels
        max_power = 0
        for pdo in message.data_objects:
            if hasattr(pdo, "max_power_w") and pdo.max_power_w:
                max_power = max(max_power, pdo.max_power_w)

        # Modern adapters should offer significant power
        assert max_power > 10  # At least 10W

    def test_protocol_debugging(self):
        """Test using the library for protocol debugging"""
        # Parse a message and examine details
        test_bytes = bytes.fromhex(
            "a1612c9101082cd102002cc103002cb10400454106003c21dcc0"
        )
        message = usbpdpy.parse_message(test_bytes)

        # Should provide detailed header information for debugging
        header = message.header
        assert hasattr(header, "message_id")  # For tracking message sequences
        assert hasattr(header, "spec_revision")  # For protocol version compatibility

        # Should provide message type name for human-readable debugging
        type_name = usbpdpy.get_message_type_name(
            header.message_type_raw, header.num_data_objects
        )
        assert isinstance(type_name, str)
        assert len(type_name) > 0

    def test_hex_utilities_for_debugging(self):
        """Test hex utilities for debug output"""
        original_hex = "a1612c91"

        # Convert to bytes and back
        byte_list = usbpdpy.hex_to_bytes(original_hex)
        converted_hex = usbpdpy.bytes_to_hex(bytes(byte_list))

        # Should round-trip correctly
        assert original_hex.lower() == converted_hex.lower()
