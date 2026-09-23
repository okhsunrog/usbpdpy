"""
Test usbpdpy with real USB PD packets from SQLite database
"""

import pytest
import usbpdpy


# Real USB PD packets extracted from pd_new.sqlite
REAL_PD_PACKETS = [
    # Source Capabilities messages (type=1, ndo=6, len=26)
    {
        "hex": "a1612c9101082cd102002cc103002cb10400454106003c21dcc0",
        "type": 1,
        "ndo": 6,
        "length": 26,
        "expected": "Source_Capabilities",
        "description": "Source Capabilities with 6 PDOs (first variant)",
    },
    {
        "hex": "a1632c9101082cd102002cc103002cb10400454106003c21dcc0",
        "type": 1,
        "ndo": 6,
        "length": 26,
        "expected": "Source_Capabilities",
        "description": "Source Capabilities with 6 PDOs (second variant)",
    },
    # GoodCRC messages (type=1, ndo=0, len=2)
    {
        "hex": "4102",
        "type": 1,
        "ndo": 0,
        "length": 2,
        "expected": "GoodCRC",
        "description": "GoodCRC acknowledgment",
    },
    {
        "hex": "2101",
        "type": 1,
        "ndo": 0,
        "length": 2,
        "expected": "GoodCRC",
        "description": "GoodCRC acknowledgment (variant 2)",
    },
    {
        "hex": "4104",
        "type": 1,
        "ndo": 0,
        "length": 2,
        "expected": "GoodCRC",
        "description": "GoodCRC acknowledgment (variant 3)",
    },
    {
        "hex": "4106",
        "type": 1,
        "ndo": 0,
        "length": 2,
        "expected": "GoodCRC",
        "description": "GoodCRC acknowledgment (variant 4)",
    },
    # Request message (type=2, ndo=1, len=6)
    {
        "hex": "8210dc700323",
        "type": 2,
        "ndo": 1,
        "length": 6,
        "expected": "Request",
        "description": "Power Request message",
    },
    # Accept message (type=3, ndo=0, len=2)
    {
        "hex": "a305",
        "type": 3,
        "ndo": 0,
        "length": 2,
        "expected": "Accept",
        "description": "Accept message",
    },
    # PS_RDY message (type=6, ndo=0, len=2)
    {
        "hex": "a607",
        "type": 6,
        "ndo": 0,
        "length": 2,
        "expected": "PS_RDY",
        "description": "Power Supply Ready message",
    },
]


class TestRealPDPackets:
    """Test parsing of real USB PD packets from SQLite database"""

    @pytest.mark.parametrize("packet", REAL_PD_PACKETS)
    def test_real_packet_parsing(self, packet):
        """Test parsing each real PD packet"""
        hex_data = packet["hex"]
        expected_type = packet["type"]
        expected_ndo = packet["ndo"]
        expected_name = packet["expected"]
        description = packet["description"]

        # Convert hex to bytes
        packet_bytes = bytes.fromhex(hex_data)

        # Parse the message
        message = usbpdpy.parse_message(packet_bytes)

        # Verify header parsing
        assert message.header.message_type_raw == expected_type, (
            f"Wrong type for {description}"
        )
        assert message.header.num_data_objects == expected_ndo, (
            f"Wrong NDO count for {description}"
        )

        # Verify message type interpretation
        interpreted_name = usbpdpy.get_message_type_name(
            message.header.message_type_raw, message.header.num_data_objects
        )
        assert interpreted_name == expected_name, (
            f"Wrong interpretation for {description}: got {interpreted_name}, expected {expected_name}"
        )

        # Verify data objects count matches header expectations
        if expected_name == "Source_Capabilities":
            assert len(message.data_objects) == expected_ndo, (
                f"Data objects count mismatch for {description}"
            )
            assert len(message.request_objects) == 0, (
                "Source Capabilities don't have RDOs"
            )
        elif expected_name == "Request":
            assert len(message.data_objects) == 0, "Request messages don't have PDOs"
            assert len(message.request_objects) == 0, (
                "Without PDO state, RDOs can't be parsed"
            )
        else:
            # Control messages
            assert len(message.data_objects) == 0, (
                "Control messages don't have data objects"
            )
            assert len(message.request_objects) == 0, (
                "Control messages don't have request objects"
            )

        print(f"✅ {description}: {hex_data} -> {interpreted_name}")


class TestSourceCapabilitiesDetails:
    """Detailed tests for Source Capabilities messages"""

    def test_source_caps_power_analysis(self):
        """Test detailed power analysis of Source Capabilities"""
        # Use the first Source Capabilities packet
        source_caps_hex = "a1612c9101082cd102002cc103002cb10400454106003c21dcc0"
        message = usbpdpy.parse_message(bytes.fromhex(source_caps_hex))

        # Should be identified as Source Capabilities
        assert message.header.message_type == "Source_Capabilities"
        assert message.is_source_capabilities()
        assert message.is_data_message()

        # Should have 6 PDOs
        assert len(message.data_objects) == 6

        # Analyze each PDO
        pdos = message.data_objects

        # PDO 1: Should be 5V fixed supply
        assert pdos[0].voltage_v == 5.0
        assert pdos[0].max_current_a == 3.0
        assert pdos[0].pdo_type == "FixedSupply"

        # PDO 2: Should be 9V fixed supply
        assert pdos[1].voltage_v == 9.0
        assert pdos[1].max_current_a == 3.0
        assert pdos[1].pdo_type == "FixedSupply"

        # PDO 3: Should be 12V fixed supply
        assert pdos[2].voltage_v == 12.0
        assert pdos[2].max_current_a == 3.0
        assert pdos[2].pdo_type == "FixedSupply"

        # PDO 4: Should be 15V fixed supply
        assert pdos[3].voltage_v == 15.0
        assert pdos[3].max_current_a == 3.0
        assert pdos[3].pdo_type == "FixedSupply"

        # PDO 5: Should be 20V fixed supply with higher current
        assert pdos[4].voltage_v == 20.0
        assert pdos[4].max_current_a == 3.25
        assert pdos[4].pdo_type == "FixedSupply"

        # PDO 6: Should be PPS (Programmable Power Supply)
        assert pdos[5].pdo_type == "PPS"
        assert pdos[5].voltage_v is None  # PPS uses voltage range
        assert pdos[5].max_current_a == 3.0

        # Calculate total maximum power capability
        max_power = max(pdo.max_power_w for pdo in pdos if pdo.max_power_w)
        assert max_power == 65.0  # 20V * 3.25A = 65W

        print(
            f"✅ Power adapter supports up to {max_power}W with {len(pdos)} power profiles"
        )


class TestRequestMessage:
    """Test Request message parsing"""

    def test_request_message_parsing(self):
        """Test parsing the Request message"""
        request_hex = "8210dc700323"
        message = usbpdpy.parse_message(bytes.fromhex(request_hex))

        # Should be identified as Request
        assert message.header.message_type == "Request"
        assert message.header.message_type_raw == 2
        assert message.header.num_data_objects == 1

        # Verify message type interpretation
        interpreted_name = usbpdpy.get_message_type_name(
            message.header.message_type_raw, message.header.num_data_objects
        )
        assert interpreted_name == "Request"

        # Request messages don't have PDOs, they have RDOs
        # Without PDO state, request_objects will be empty, but header shows the structure
        assert len(message.data_objects) == 0  # Request messages don't have PDOs
        assert (
            len(message.request_objects) == 0
        )  # Without PDO state, RDOs can't be parsed

        print(f"✅ Request message parsed: {request_hex}")

    def test_request_message_with_pdo_state(self):
        """Test Request message parsing with PDO state"""
        # Parse Source Capabilities first
        source_caps_hex = "a1612c9101082cd102002cc103002cb10400454106003c21dcc0"
        source_msg = usbpdpy.parse_message(bytes.fromhex(source_caps_hex))

        # Parse Request with PDO state
        request_hex = "8210dc700323"
        request_msg = usbpdpy.parse_pd_message_with_state(
            bytes.fromhex(request_hex), source_msg.data_objects
        )

        # Should now have Request Data Objects parsed
        assert request_msg.header.message_type == "Request"
        assert len(request_msg.request_objects) == 1

        # Check RDO details
        rdo = request_msg.request_objects[0]
        assert rdo.object_position == 2  # Requesting PDO #2
        assert rdo.raw == 0x230370DC

        print(f"✅ Request with PDO state: requesting PDO #{rdo.object_position}")


# Source: km003c-protocol-research, capture pd_epr0.9 - a sink requesting the
# 28 V / 5 A (140 W) EPR fixed PDO at object position 8.
EPR_REQUEST_HEX = "8926f4d14781f4c10800"


class TestEprRequest:
    """EPR_Request carries a copy of the PDO it selects, so it is self-describing."""

    def assert_epr_fixed_request(self, message):
        assert message.header.message_type == "EPR_Request"
        assert len(message.request_objects) == 1

        rdo = message.request_objects[0]
        assert rdo.rdo_type == "EPR_FixedVariableSupply"
        assert rdo.object_position == 8
        assert rdo.operating_current_a == pytest.approx(5.0)
        assert rdo.max_operating_current_a == pytest.approx(5.0)

    def test_decodes_without_source_capabilities(self):
        # A plain Request needs the preceding Source_Capabilities to be
        # interpreted; an EPR_Request does not, so the stateless parser can
        # and should decode it.
        self.assert_epr_fixed_request(
            usbpdpy.parse_pd_message(bytes.fromhex(EPR_REQUEST_HEX))
        )

    def test_decodes_with_the_stateful_parser_too(self):
        self.assert_epr_fixed_request(
            usbpdpy.parse_pd_message_with_state(bytes.fromhex(EPR_REQUEST_HEX), None)
        )


# Source: km003c-protocol-research, capture pd_epr0.9 frame 1099 - a 32-byte
# EPR_Source_Capabilities sent as a 26-byte chunk, the sink's request for chunk
# 1, and the final 6-byte chunk.
EPR_SOURCE_CAPS_CHUNKS = [
    "b1fb20802c91812b2cd102002cc103002cb10400f44106006421a4c90000",
    "9194008c0000",
    "b1ad20880000f4c10800",
]


class TestPdDecoder:
    """PdDecoder keeps the state that single-message parsing lacks."""

    def test_resolves_a_request_against_the_preceding_capabilities(self):
        decoder = usbpdpy.PdDecoder()

        caps = decoder.decode(
            bytes.fromhex("a1612c9101082cd102002cc103002cb10400454106003c21dcc0")
        )
        request = decoder.decode(bytes.fromhex("8210dc700323"))

        assert len(decoder.source_capabilities) == len(caps.data_objects) == 6
        assert request.header.message_type == "Request"
        assert len(request.request_objects) == 1
        assert request.request_objects[0].object_position == 2
        assert request.request_objects[0].raw == 0x230370DC

    def test_leaves_a_request_unresolved_before_any_capabilities(self):
        request = usbpdpy.PdDecoder().decode(bytes.fromhex("8210dc700323"))

        assert request.header.message_type == "Request"
        assert request.request_objects == []

    def test_reassembles_chunked_epr_source_capabilities(self):
        decoder = usbpdpy.PdDecoder()

        first, chunk_request, last = (
            decoder.decode(bytes.fromhex(chunk)) for chunk in EPR_SOURCE_CAPS_CHUNKS
        )

        assert first is None
        assert chunk_request is None
        assert last.header.message_type == "EPR_Source_Capabilities"
        assert last.raw_bytes == bytes.fromhex(EPR_SOURCE_CAPS_CHUNKS[-1])
        # Positions 1-7 hold the SPR PDOs, zero-padded; EPR PDOs start at 8.
        pdos = last.data_objects
        assert [pdo.raw for pdo in pdos[5:]] == [0xC9A42164, 0, 0x0008C1F4]
        assert pdos[0].voltage_v == pytest.approx(5.0)
        assert pdos[7].voltage_v == pytest.approx(28.0)
        assert pdos[7].max_current_a == pytest.approx(5.0)

    def test_single_message_parsing_cannot_decode_a_chunk(self):
        with pytest.raises(ValueError, match="ChunkedExtendedMessage"):
            usbpdpy.parse_pd_message(bytes.fromhex(EPR_SOURCE_CAPS_CHUNKS[0]))

    def test_a_new_first_chunk_restarts_assembly(self):
        decoder = usbpdpy.PdDecoder()
        first, _, last = (bytes.fromhex(chunk) for chunk in EPR_SOURCE_CAPS_CHUNKS)

        assert decoder.decode(first) is None
        assert decoder.decode(first) is None  # retransmitted after an abort
        assert decoder.decode(last).header.message_type == "EPR_Source_Capabilities"

    def test_rejects_a_chunk_without_its_first_chunk(self):
        decoder = usbpdpy.PdDecoder()

        with pytest.raises(ValueError):
            decoder.decode(bytes.fromhex(EPR_SOURCE_CAPS_CHUNKS[-1]))

    def test_reset_forgets_the_capabilities(self):
        decoder = usbpdpy.PdDecoder()
        decoder.decode(
            bytes.fromhex("a1612c9101082cd102002cc103002cb10400454106003c21dcc0")
        )

        decoder.reset()

        assert decoder.source_capabilities == []
        assert decoder.decode(bytes.fromhex("8210dc700323")).request_objects == []


class TestControlMessages:
    """Test various control messages"""

    def test_control_messages(self):
        """Test parsing of various control messages"""
        control_messages = [
            ("a305", 3, "Accept"),
            ("a607", 6, "PS_RDY"),
        ]

        for hex_data, expected_type, expected_name in control_messages:
            message = usbpdpy.parse_message(bytes.fromhex(hex_data))

            # Should be control message (no data objects)
            assert message.header.num_data_objects == 0
            assert message.is_control_message()
            assert not message.is_data_message()

            # Verify type
            assert message.header.message_type_raw == expected_type

            # Verify interpretation
            interpreted_name = usbpdpy.get_message_type_name(
                message.header.message_type_raw, message.header.num_data_objects
            )
            assert interpreted_name == expected_name

            print(f"✅ Control message {expected_name}: {hex_data}")


class TestCoreBugFix:
    """Test the core bug fix with real data"""

    def test_source_caps_vs_goodcrc_real_data(self):
        """Test that Source Capabilities and GoodCRC are distinguished using real packets"""

        # Real Source Capabilities packet
        source_caps_hex = "a1612c9101082cd102002cc103002cb10400454106003c21dcc0"
        source_caps_bytes = bytes.fromhex(source_caps_hex)

        # Real GoodCRC packet
        goodcrc_hex = "4102"
        goodcrc_bytes = bytes.fromhex(goodcrc_hex)

        # Parse both
        source_msg = usbpdpy.parse_message(source_caps_bytes)
        goodcrc_msg = usbpdpy.parse_message(goodcrc_bytes)

        # Both have message_type_raw == 1, but different data object counts
        assert source_msg.header.message_type_raw == 1
        assert goodcrc_msg.header.message_type_raw == 1

        assert source_msg.header.num_data_objects == 6
        assert goodcrc_msg.header.num_data_objects == 0

        # Verify correct interpretation
        source_name = usbpdpy.get_message_type_name(
            source_msg.header.message_type_raw, source_msg.header.num_data_objects
        )
        goodcrc_name = usbpdpy.get_message_type_name(
            goodcrc_msg.header.message_type_raw, goodcrc_msg.header.num_data_objects
        )

        # The critical test - this was the original bug
        assert source_name == "Source_Capabilities", (
            f"Source Capabilities incorrectly labeled as {source_name}"
        )
        assert goodcrc_name == "GoodCRC", (
            f"GoodCRC incorrectly labeled as {goodcrc_name}"
        )

        # Also verify string representation
        assert source_msg.header.message_type == "Source_Capabilities"
        assert goodcrc_msg.header.message_type == "GoodCRC"

        print("✅ CORE BUG FIX VERIFIED WITH REAL DATA:")
        print(f"   Source Capabilities: {source_caps_hex} -> {source_name}")
        print(f"   GoodCRC: {goodcrc_hex} -> {goodcrc_name}")


class TestPacketHexUtils:
    """Test hex utility functions with real packet data"""

    def test_hex_roundtrip_real_packets(self):
        """Test hex conversion with real packet data"""
        test_packets = [
            "a1612c9101082cd102002cc103002cb10400454106003c21dcc0",
            "4102",
            "8210dc700323",
            "a305",
            "a607",
        ]

        for hex_str in test_packets:
            # Convert to bytes and back
            packet_bytes = usbpdpy.hex_to_bytes(hex_str)
            hex_back = usbpdpy.bytes_to_hex(packet_bytes)

            # Should round-trip correctly
            assert hex_back.lower() == hex_str.lower()

            # Should parse successfully
            message = usbpdpy.parse_message(packet_bytes)
            assert message is not None

            print(f"✅ Hex round-trip: {hex_str}")
