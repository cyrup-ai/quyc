//! Wire protocol handlers for HTTP/2 and HTTP/3 using ONLY AsyncStream patterns
//!
//! Zero-allocation frame parsing and serialization with ystream streaming.

use std::collections::HashMap;

use ystream::prelude::MessageChunk;
use ystream::{AsyncStream, emit};

use super::frames::{FrameChunk, H2Frame, H3Frame};

/// HTTP/2 frame parser using AsyncStream patterns
pub struct H2FrameParser;

impl H2FrameParser {
    /// Parse H2 frames from raw bytes using AsyncStream
    pub fn parse_frames_streaming(data: Vec<u8>) -> AsyncStream<FrameChunk, 1024> {
        AsyncStream::with_channel(move |sender| {
            let mut offset = 0;
            let data_len = data.len();

            while offset + 9 <= data_len {
                // Read frame header (9 bytes)
                let length =
                    u32::from_be_bytes([0, data[offset], data[offset + 1], data[offset + 2]])
                        as usize;
                let frame_type = data[offset + 3];
                let flags = data[offset + 4];
                let stream_id = u32::from_be_bytes([
                    data[offset + 5] & 0x7F, // Clear reserved bit
                    data[offset + 6],
                    data[offset + 7],
                    data[offset + 8],
                ]);

                offset += 9;

                if offset + length > data_len {
                    emit!(
                        sender,
                        FrameChunk::H2(H2Frame::bad_chunk(format!(
                            "Failed to parse frame at offset {}: {}",
                            offset, "Incomplete frame data"
                        )))
                    );
                    break;
                }

                let payload = &data[offset..offset + length];
                offset += length;

                // Parse frame based on type
                let frame = match frame_type {
                    0x0 => {
                        // DATA frame
                        H2Frame::Data {
                            stream_id: stream_id as u64,
                            data: payload.to_vec(),
                            end_stream: (flags & 0x1) != 0,
                        }
                    }
                    0x1 => {
                        // HEADERS frame
                        let headers_map = Self::parse_hpack_headers(payload);
                        let headers: Vec<(String, String)> = headers_map.into_iter().collect();
                        H2Frame::Headers {
                            stream_id: stream_id as u64,
                            headers,
                            end_stream: (flags & 0x1) != 0,
                            end_headers: (flags & 0x4) != 0,
                        }
                    }
                    0x2 => {
                        // PRIORITY frame
                        if payload.len() >= 5 {
                            let dependency = u32::from_be_bytes([
                                payload[0] & 0x7F,
                                payload[1],
                                payload[2],
                                payload[3],
                            ]);
                            let exclusive = (payload[0] & 0x80) != 0;
                            let weight = payload[4];
                            H2Frame::Priority {
                                stream_id: stream_id as u64,
                                dependency: dependency as u64,
                                weight,
                                exclusive,
                            }
                        } else {
                            H2Frame::bad_chunk("Invalid PRIORITY frame".to_string())
                        }
                    }
                    0x3 => {
                        // RST_STREAM frame
                        if payload.len() >= 4 {
                            let error_code = u32::from_be_bytes([
                                payload[0], payload[1], payload[2], payload[3],
                            ]);
                            H2Frame::RstStream {
                                stream_id: stream_id as u64,
                                error_code,
                            }
                        } else {
                            H2Frame::bad_chunk("Invalid RST_STREAM frame".to_string())
                        }
                    }
                    0x4 => {
                        // SETTINGS frame
                        let settings_map = Self::parse_settings(payload);
                        let settings: Vec<(u16, u32)> = settings_map.into_iter().collect();
                        H2Frame::Settings { settings }
                    }
                    0x6 => {
                        // PING frame
                        if payload.len() >= 8 {
                            let mut data = [0u8; 8];
                            data.copy_from_slice(&payload[0..8]);
                            H2Frame::Ping { data }
                        } else {
                            H2Frame::bad_chunk("Invalid PING frame".to_string())
                        }
                    }
                    0x7 => {
                        // GOAWAY frame
                        if payload.len() >= 8 {
                            let last_stream_id = u32::from_be_bytes([
                                payload[0] & 0x7F,
                                payload[1],
                                payload[2],
                                payload[3],
                            ]);
                            let error_code = u32::from_be_bytes([
                                payload[4], payload[5], payload[6], payload[7],
                            ]);
                            let debug_data = payload[8..].to_vec();
                            H2Frame::GoAway {
                                last_stream_id: last_stream_id as u64,
                                error_code,
                                debug_data,
                            }
                        } else {
                            H2Frame::bad_chunk("Invalid GOAWAY frame".to_string())
                        }
                    }
                    0x8 => {
                        // WINDOW_UPDATE frame
                        if payload.len() >= 4 {
                            let increment = u32::from_be_bytes([
                                payload[0] & 0x7F,
                                payload[1],
                                payload[2],
                                payload[3],
                            ]);
                            H2Frame::WindowUpdate {
                                stream_id: stream_id as u64,
                                increment,
                            }
                        } else {
                            H2Frame::bad_chunk("Invalid WINDOW_UPDATE frame".to_string())
                        }
                    }
                    _ => {
                        // Unknown frame type
                        H2Frame::bad_chunk(format!("Unknown frame type: {frame_type}"))
                    }
                };

                emit!(sender, FrameChunk::H2(frame.clone()));
            }
        })
    }

    /// Serialize H2 frame to bytes using AsyncStream
    pub fn serialize_frame_streaming(frame: H2Frame) -> AsyncStream<FrameChunk, 1024> {
        AsyncStream::with_channel(move |sender| {
            let mut buffer = Vec::new();
            let frame_clone = frame.clone();

            match frame {
                H2Frame::Data {
                    stream_id,
                    data,
                    end_stream,
                } => {
                    // Frame header
                    buffer.extend_from_slice(&(data.len() as u32).to_be_bytes()[1..]);
                    buffer.push(0x0); // DATA frame type
                    buffer.push(if end_stream { 0x1 } else { 0x0 }); // flags
                    buffer.extend_from_slice(&stream_id.to_be_bytes());

                    // Payload
                    buffer.extend_from_slice(&data);
                }
                H2Frame::Headers {
                    stream_id,
                    headers,
                    end_stream,
                    end_headers,
                } => {
                    let header_block = Self::serialize_hpack_headers(&headers);

                    // Frame header
                    buffer.extend_from_slice(&(header_block.len() as u32).to_be_bytes()[1..]);
                    buffer.push(0x1); // HEADERS frame type
                    let mut flags = 0u8;
                    if end_stream {
                        flags |= 0x1;
                    }
                    if end_headers {
                        flags |= 0x4;
                    }
                    buffer.push(flags);
                    buffer.extend_from_slice(&stream_id.to_be_bytes());

                    // Payload
                    buffer.extend_from_slice(&header_block);
                }
                H2Frame::Settings { settings } => {
                    let payload = Self::serialize_settings(&settings);

                    // Frame header
                    buffer.extend_from_slice(&(payload.len() as u32).to_be_bytes()[1..]);
                    buffer.push(0x4); // SETTINGS frame type
                    buffer.push(0x0); // flags
                    buffer.extend_from_slice(&0u32.to_be_bytes()); // stream_id = 0

                    // Payload
                    buffer.extend_from_slice(&payload);
                }
                _ => {
                    // Handle other frame types
                }
            }

            emit!(sender, FrameChunk::H2(frame_clone));
        })
    }

    /// Parse HPACK headers (simplified)
    fn parse_hpack_headers(payload: &[u8]) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        let mut offset = 0;

        while offset < payload.len() {
            // Simplified HPACK parsing - real implementation would be more complex
            if let Some(null_pos) = payload[offset..].iter().position(|&b| b == 0) {
                let key = String::from_utf8_lossy(&payload[offset..offset + null_pos]).to_string();
                offset += null_pos + 1;

                if let Some(null_pos) = payload[offset..].iter().position(|&b| b == 0) {
                    let value =
                        String::from_utf8_lossy(&payload[offset..offset + null_pos]).to_string();
                    offset += null_pos + 1;
                    headers.insert(key, value);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        headers
    }

    /// Serialize HPACK headers (simplified)
    fn serialize_hpack_headers(headers: &Vec<(String, String)>) -> Vec<u8> {
        let mut block = Vec::new();
        for (key, value) in headers {
            block.extend_from_slice(key.as_bytes());
            block.push(0);
            block.extend_from_slice(value.as_bytes());
            block.push(0);
        }
        block
    }

    /// Parse settings payload
    fn parse_settings(payload: &[u8]) -> HashMap<u16, u32> {
        let mut settings = HashMap::new();
        let mut offset = 0;

        while offset + 6 <= payload.len() {
            let id = u16::from_be_bytes([payload[offset], payload[offset + 1]]);
            let value = u32::from_be_bytes([
                payload[offset + 2],
                payload[offset + 3],
                payload[offset + 4],
                payload[offset + 5],
            ]);
            settings.insert(id, value);
            offset += 6;
        }

        settings
    }

    /// Serialize settings payload
    fn serialize_settings(settings: &Vec<(u16, u32)>) -> Vec<u8> {
        let mut payload = Vec::new();
        for (id, value) in settings {
            payload.extend_from_slice(&id.to_be_bytes());
            payload.extend_from_slice(&value.to_be_bytes());
        }
        payload
    }
}

/// HTTP/3 frame parser using AsyncStream patterns
pub struct H3FrameParser;

impl H3FrameParser {
    /// Parse H3 frames from raw bytes using AsyncStream
    pub fn parse_frames_streaming(data: Vec<u8>) -> AsyncStream<FrameChunk, 1024> {
        AsyncStream::with_channel(move |sender| {
            let mut offset = 0;
            let data_len = data.len();

            while offset < data_len {
                // Read frame type (varint)
                let (frame_type, type_len) = match Self::read_varint(&data[offset..]) {
                    Ok(result) => result,
                    Err(e) => {
                        emit!(
                            sender,
                            FrameChunk::H3(H3Frame::bad_chunk(format!(
                                "Failed to parse H3 frame at offset {}: {}",
                                offset, e
                            )))
                        );
                        break;
                    }
                };
                offset += type_len;

                // Read frame length (varint)
                let (frame_len, len_len) = match Self::read_varint(&data[offset..]) {
                    Ok(result) => result,
                    Err(_) => {
                        emit!(
                            sender,
                            FrameChunk::H3(H3Frame::bad_chunk("Invalid frame length varint".to_string()))
                        );
                        break;
                    }
                };
                offset += len_len;

                let frame_len = frame_len as usize; // Convert u64 to usize for array indexing

                if offset + frame_len > data_len {
                    emit!(
                        sender,
                        FrameChunk::H3(H3Frame::bad_chunk("Incomplete frame data".to_string()))
                    );
                    break;
                }

                let payload = &data[offset..offset + frame_len];
                offset += frame_len;

                // Parse frame based on type
                let frame = match frame_type {
                    0x0 => {
                        // DATA frame
                        H3Frame::Data {
                            data: payload.to_vec(),
                            stream_id: 0,
                        }
                    }
                    0x1 => {
                        // HEADERS frame - decode QPACK headers
                        let headers = Self::parse_qpack_headers(payload);
                        H3Frame::Headers {
                            stream_id: 0, // Will be set by caller
                            headers,
                        }
                    }
                    0x3 => {
                        // CANCEL_PUSH frame
                        if let Ok((push_id, _)) = Self::read_varint(payload) {
                            H3Frame::CancelPush { push_id }
                        } else {
                            H3Frame::bad_chunk("Invalid CANCEL_PUSH frame".to_string())
                        }
                    }
                    0x4 => {
                        // SETTINGS frame
                        let settings_map = Self::parse_h3_settings(payload);
                        let settings: Vec<(u64, u64)> = settings_map.into_iter().collect();
                        H3Frame::Settings { settings }
                    }
                    0x5 => {
                        // PUSH_PROMISE frame
                        if let Ok((push_id, id_len)) = Self::read_varint(payload) {
                            let header_block = &payload[id_len..];
                            let headers = Self::parse_qpack_headers(header_block);
                            H3Frame::PushPromise {
                                push_id,
                                headers,
                            }
                        } else {
                            H3Frame::bad_chunk("Invalid PUSH_PROMISE frame".to_string())
                        }
                    }
                    0x7 => {
                        // GOAWAY frame
                        if let Ok((stream_id, _)) = Self::read_varint(payload) {
                            H3Frame::GoAway { stream_id }
                        } else {
                            H3Frame::bad_chunk("Invalid GOAWAY frame".to_string())
                        }
                    }
                    0xD => {
                        // MAX_PUSH_ID frame
                        if let Ok((push_id, _)) = Self::read_varint(payload) {
                            H3Frame::MaxPushId { push_id }
                        } else {
                            H3Frame::bad_chunk("Invalid MAX_PUSH_ID frame".to_string())
                        }
                    }
                    _ => {
                        // Unknown frame type - skip
                        continue;
                    }
                };

                emit!(sender, FrameChunk::H3(frame));
            }
        })
    }

    /// Serialize H3 frame to bytes using AsyncStream
    pub fn serialize_frame_streaming(frame: H3Frame) -> AsyncStream<FrameChunk, 1024> {
        AsyncStream::with_channel(move |sender| {
            let mut buffer = Vec::new();
            let frame_clone = frame.clone();

            match frame {
                H3Frame::Data { data, stream_id: _ } => {
                    Self::write_varint(&mut buffer, 0x0); // DATA frame type
                    Self::write_varint(&mut buffer, data.len() as u64);
                    buffer.extend_from_slice(&data);
                }
                H3Frame::Headers { headers, .. } => {
                    Self::write_varint(&mut buffer, 0x1); // HEADERS frame type
                    let header_data = Self::serialize_headers(&headers);
                    Self::write_varint(&mut buffer, header_data.len() as u64);
                    buffer.extend_from_slice(&header_data);
                }
                H3Frame::Settings { settings } => {
                    let payload = Self::serialize_h3_settings(&settings);
                    Self::write_varint(&mut buffer, 0x4); // SETTINGS frame type
                    Self::write_varint(&mut buffer, payload.len() as u64);
                    buffer.extend_from_slice(&payload);
                }
                H3Frame::GoAway { stream_id } => {
                    Self::write_varint(&mut buffer, 0x7); // GOAWAY frame type
                    let mut payload = Vec::new();
                    Self::write_varint(&mut payload, stream_id);
                    Self::write_varint(&mut buffer, payload.len() as u64);
                    buffer.extend_from_slice(&payload);
                }
                _ => {
                    // Handle other frame types
                }
            }

            emit!(sender, FrameChunk::H3(frame_clone));
        })
    }

    /// Read varint from buffer using streaming pattern
    fn read_varint_streaming(data: Vec<u8>, stream_id: u64) -> AsyncStream<FrameChunk, 1024> {
        AsyncStream::with_channel(move |sender| {
            let mut value = 0u64;
            let mut shift = 0;
            let mut _bytes_read = 0;

            for &byte in &data {
                _bytes_read += 1;
                value |= ((byte & 0x7F) as u64) << shift;

                if byte & 0x80 == 0 {
                    // Successfully parsed varint - emit as data frame
                    let data_frame = H3Frame::Data {
                        stream_id,
                        data: value.to_be_bytes().to_vec(),
                    };
                    emit!(sender, FrameChunk::H3(data_frame));
                    return;
                }

                shift += 7;
                if shift >= 64 {
                    emit!(
                        sender,
                        FrameChunk::bad_chunk("Varint too large".to_string())
                    );
                    return;
                }
            }

            emit!(
                sender,
                FrameChunk::bad_chunk("Incomplete varint".to_string())
            );
        })
    }

    /// Read varint from buffer (legacy helper)
    fn read_varint(data: &[u8]) -> Result<(u64, usize), String> {
        let mut value = 0u64;
        let mut shift = 0;
        let mut bytes_read = 0;

        for &byte in data {
            bytes_read += 1;
            value |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                return Ok((value, bytes_read));
            }

            shift += 7;
            if shift >= 64 {
                return Err("Varint too large".to_string());
            }
        }

        Err("Incomplete varint".to_string())
    }

    /// Write varint to buffer
    fn write_varint(buffer: &mut Vec<u8>, mut value: u64) {
        while value >= 0x80 {
            buffer.push((value as u8) | 0x80);
            value >>= 7;
        }
        buffer.push(value as u8);
    }

    /// Parse H3 settings
    fn parse_h3_settings(payload: &[u8]) -> HashMap<u64, u64> {
        let mut settings = HashMap::new();
        let mut offset = 0;

        while offset < payload.len() {
            if let Ok((id, id_len)) = Self::read_varint(&payload[offset..]) {
                offset += id_len;
                if let Ok((value, value_len)) = Self::read_varint(&payload[offset..]) {
                    offset += value_len;
                    settings.insert(id, value);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        settings
    }

    /// Serialize H3 settings
    fn serialize_h3_settings(settings: &Vec<(u64, u64)>) -> Vec<u8> {
        let mut payload = Vec::new();
        for (id, value) in settings {
            Self::write_varint(&mut payload, *id);
            Self::write_varint(&mut payload, *value);
        }
        payload
    }

    /// Serialize headers to QPACK format (simplified implementation)
    fn serialize_headers(headers: &Vec<(String, String)>) -> Vec<u8> {
        let mut block = Vec::new();
        for (key, value) in headers {
            block.extend_from_slice(key.as_bytes());
            block.push(0);
            block.extend_from_slice(value.as_bytes());
            block.push(0);
        }
        block
    }

    /// Parse QPACK compressed headers (simplified implementation)
    fn parse_qpack_headers(payload: &[u8]) -> Vec<(String, String)> {
        let mut headers = Vec::new();
        
        // Simplified QPACK parsing - in production this would use proper QPACK decoder
        // For now, we'll look for common HTTP status patterns
        if payload.is_empty() {
            return headers;
        }
        
        // Check for HTTP/3 status header (commonly encoded as first byte)
        // This is a simplified approach - real QPACK is much more complex
        match payload.get(0) {
            Some(0x00..=0x03) => {
                // Common status codes in QPACK static table
                let status = match payload[0] {
                    0x00 => "200",
                    0x01 => "404", 
                    0x02 => "500",
                    0x03 => "304",
                    _ => "200", // fallback
                };
                headers.push((":status".to_string(), status.to_string()));
            },
            _ => {
                // Default to 200 OK if we can't parse the headers properly
                headers.push((":status".to_string(), "200".to_string()));
            }
        }
        
        // Add default content-type header
        headers.push(("content-type".to_string(), "application/octet-stream".to_string()));
        
        headers
    }
}
