use std::io::{self, BufRead, Write};

use anyhow::Context;
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
    Echo { echo: String },
    EchoOk { echo: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Init {
    node_id: String,
    node_ids: Vec<String>,
}

impl Kind {
    fn into_response(self) -> Kind {
        match self {
            Kind::Init(_) => Kind::InitOk,
            Kind::Echo { echo } => Kind::EchoOk { echo },
            _ => unimplemented!(),
        }
    }
}

impl Message {
    fn into_response(self, output: &mut impl io::Write) -> anyhow::Result<()> {
        let response = Message {
            src: self.dst,
            dst: self.src,
            body: Body {
                msg_id: Some(0),
                in_reply_to: self.body.msg_id,
                kind: self.body.kind.into_response(),
            },
        };
        serde_json::to_writer(&mut *output, &response).context("serialize response")?;
        output
            .write_all(b"\n")
            .context("write new line to buffer")?;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut stdin = std::io::stdin().lock().lines();
    let mut stdout = std::io::stdout().lock();

    let init_message =
        serde_json::from_str::<Message>(&stdin.next().expect("no init message received")?)?;

    init_message
        .into_response(&mut stdout)
        .context("response")?;

    for line in stdin {
        let message =
            serde_json::from_str::<Message>(&line.context("Maelstrom input could not be read")?)
                .context("maelstrom input could not be deserialize")?;

        message.into_response(&mut stdout).context("response")?;
    }

    Ok(())
}
