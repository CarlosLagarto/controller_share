use std::hash::*;

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Subscriber {
    Main = 0,
    Mqtt = 1,
    Test = 2,
    Web = 3,
    Dev = 4,
}
pub const MAX_SUBSCRIBERS: usize = 4; //Actually (13/Jun/2022) we don't have > 2 subscribers by message type.  If/when needed change here

impl Subscriber {
    #[inline]
    pub fn to_string<'a>(&self) -> &'a str {
        match *self {
            Subscriber::Main => "Main",
            Subscriber::Mqtt => "MQTT",
            Subscriber::Test => "Test",
            Subscriber::Web => "Web",
            Subscriber::Dev => "Dev",
        }
    }
}
