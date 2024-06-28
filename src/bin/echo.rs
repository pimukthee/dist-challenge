use anyhow::Context;
use dist_challenge::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoBody {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode {
    id: String,
}

impl Node<EchoBody> for EchoNode {
    fn new(id: String) -> Self {
        Self { id }
    }

    fn handle(&self, message: Message<EchoBody>, output: &mut impl Write) -> anyhow::Result<()> {
        let mut response = message.into_response();
        match response.body.kind {
            EchoBody::Echo { echo } => response.body.kind = EchoBody::EchoOk { echo },
            EchoBody::EchoOk { .. } => {}
        };

        serde_json::to_writer(&mut *output, &response).context("failed to serialize echo")?;
        output.write_all(b"\n")?;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    start_node::<EchoNode, EchoBody>()?;
    Ok(())
}
