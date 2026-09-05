pub mod order;


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum MsgType {
	Invalid = 0,
	Order = 1,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct MsgHeader {
	pub msg_len: usize,
	pub msg_type: MsgType,
}


impl MsgHeader {
		pub fn from_bytes(bytes: &[u8]) -> Result<MsgHeader, &'static str> {
			if bytes.len() < size_of::<MsgHeader>() {
				return Err("not enough bytes to read from header");
			}

			let msg_len = usize::from_ne_bytes(bytes[0..8].try_into().unwrap());
			let msg_type = match bytes[8] {
				1 => MsgType::Order,
				_ => MsgType::Invalid,
			};

			Ok(MsgHeader {msg_len, msg_type} )
		}
}