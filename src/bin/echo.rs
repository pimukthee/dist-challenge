use anyhow::Context;
use dist_challenge::*;
use serde::{Deserialize, Serialize};
use std::io::BufRead;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoBody {
    Echo { echo: String },
    EchoOk { echo: String },
}

impl IntoResponse for EchoBody {
    fn into_response(self) -> EchoBody {
        match self {
            EchoBody::Echo { echo } => EchoBody::EchoOk { echo },
            _ => unimplemented!(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut stdin = std::io::stdin().lock().lines();
    let mut stdout = std::io::stdout().lock();

    let init_message = serde_json::from_str::<Message<InitBody>>(
        &stdin.next().expect("no init message received")?,
    )?;

    init_message
        .into_response(&mut stdout)
        .context("response")?;

    for line in stdin {
        let message = serde_json::from_str::<Message<EchoBody>>(
            &line.context("Maelstrom input could not be read")?,
        )
        .context("maelstrom input could not be deserialize")?;

        message.into_response(&mut stdout).context("response")?;
    }

    Ok(())
}

