use std::io::BufRead;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body {
    #[serde(flatten)]
    kind: Kind,
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Kind {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Init {
    node_id: String,
    node_ids: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let mut stdin = std::io::stdin().lock().lines();
    let mut stdout = std::io::stdout().lock();

    let init_message =
        serde_json::from_str::<Message>(&stdin.next().expect("no init message received")?)?;

    let reply = Message {
        src: init_message.dst,
        dst: init_message.src,
        body: Body {
            msg_id: Some(0),
            in_reply_to: init_message.body.msg_id,
            kind: Kind::InitOk,
        },
    };

    serde_json::to_writer(&mut stdout, &reply);

    Ok(())
}
