use dist_challenge::*;
use std::io::BufRead;
use ulid::Ulid;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum UniqueIdBody {
    Generate,
    GenerateOk { id: String },
}

impl IntoResponse for UniqueIdBody {
    fn into_response(self) -> Self {
        UniqueIdBody::GenerateOk {
            id: Ulid::new().to_string(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut lines = std::io::stdin().lock().lines();
    let mut stdout = std::io::stdout().lock();

    let init_message = serde_json::from_str::<Message<InitBody>>(
        &lines
            .next()
            .expect("not received init message")
            .context("failed to retrieved init message from stdin")?,
    )
    .context("failed to deserialize init message")?;
    init_message
        .into_response(&mut stdout)
        .context("response")?;

    for line in lines {
        let message = serde_json::from_str::<Message<UniqueIdBody>>(
            &line.context("Maelstrom input could not be read")?,
        )
        .context("maelstrom input could not be deserialize")?;

        message.into_response(&mut stdout).context("response")?;
    }

    Ok(())
}
