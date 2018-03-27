//! Message headers.
use std::fmt;
use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use error::Result;
use error::Error::ResponseError;

/// Represents an opcode in the MongoDB Wire Protocol.
#[derive(Clone, Debug)]
pub enum OpCode {
    Reply = 1,
    Query = 2004
}

impl OpCode {
    /// Maps integer values to OpCodes
    ///
    /// # Arguments
    ///
    /// `i` - The integer to map.
    ///
    /// # Return value
    ///
    /// Returns the matching opcode, or `None` if the integer isn't a valid
    /// opcode.
    pub fn from_i32(i: i32) -> Option<OpCode> {
        match i {
            1 => Some(OpCode::Reply),
            2004 => Some(OpCode::Query),
            _ => None
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OpCode::Reply => write!(fmt, "OP_REPLY"),
            OpCode::Query => write!(fmt, "OP_QUERY")
        }
    }
}

/// Represents a header in the MongoDB Wire Protocol.
#[derive(Clone, Debug)]
pub struct Header {
    /// The length of the entire message, in bytes.
    pub message_length: i32,
    /// Identifies the request being sent. From a server response, this should be '0'.
    pub request_id: i32,
    // Identifies which response this message is a response to. From a client request, this should
    // be '0'.
    response_to: i32,
    /// Identifies which type of message is being sent.
    pub op_code: OpCode
}

impl Header {
    /// Constructs a new Header.
    pub fn new(message_length: i32, request_id: i32, response_to: i32, op_code: OpCode) -> Header {
        Header {
            message_length: message_length,
            request_id: request_id,
            response_to: response_to,
            op_code: op_code
        }
    }

    /// Constructs a new Header for a request, with `response_to` set to 0.
    fn new_request(message_length: i32, request_id: i32, op_code: OpCode) -> Header {
        Header::new(message_length, request_id, 0, op_code)
    }

    /// Constructs a new Header for an OP_QUERY, with `response_to` set to 0 and
    /// `op_code` set to `Query`.
    pub fn new_query(message_length: i32, request_id: i32) -> Header {
        Header::new_request(message_length, request_id, OpCode::Query)
    }

    /// Writes the serialized Header to a buffer.
    ///
    /// # Arguments
    ///
    /// `buffer` - The buffer to write to.
    ///
    /// # Return value
    ///
    /// Returns nothing on success, or an Error on failure.
    pub fn write<W: Write>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_i32::<LittleEndian>(self.message_length)?;
        buffer.write_i32::<LittleEndian>(self.request_id)?;
        buffer.write_i32::<LittleEndian>(self.response_to)?;
        buffer.write_i32::<LittleEndian>(self.op_code.clone() as i32)?;
        let _ = buffer.flush();

        Ok(())
    }

    /// Reads a serialized Header from a buffer.
    ///
    /// # Arguments
    ///
    /// `buffer` - The buffer to read from.
    ///
    /// # Return value
    ///
    /// Returns the parsed Header on success, or an Error on failure.
    pub fn read<R: Read>(buffer: &mut R) -> Result<Header> {
        let message_length = buffer.read_i32::<LittleEndian>()?;
        let request_id = buffer.read_i32::<LittleEndian>()?;
        let response_to = buffer.read_i32::<LittleEndian>()?;

        let op_code_i32 = buffer.read_i32::<LittleEndian>()?;
        let op_code = match OpCode::from_i32(op_code_i32) {
            Some(code) => code,
            _ => {
                return Err(ResponseError(format!("Invalid header opcode from server: {}.",
                                                 op_code_i32)))
            }
        };

        Ok(Header::new(message_length, request_id, response_to, op_code))
    }
}
