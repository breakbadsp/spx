pub mod order;


pub enum MsgType {
	Order,
}

pub struct MsgHeader {
	msg_len: usize,
	msg_type: MsgType,
}