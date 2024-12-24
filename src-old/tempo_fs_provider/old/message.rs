use std::io::Read;

/*

message file layout

[8 bytes: message type, u64]
[remaining bytes: json of message]

messages are zstd compressed

 */

pub(crate) enum Message {

}

impl Message {
    pub fn load<P>(path: P) -> anyhow::Result<Self> {
        let f = std::fs::File::open(p)?;

        
    }
}