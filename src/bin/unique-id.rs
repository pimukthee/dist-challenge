use anyhow::Context;
use dist_challenge::*;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum UniqueIdBody {
    Generate,
    GenerateOk { id: String },
}

struct UniqueIdNode {
    id: String,
}

impl Node<UniqueIdBody> for UniqueIdNode {
    fn new(id: String) -> Self {
        Self { id }
    }

    fn handle(
        &mut self,
        message: Message<UniqueIdBody>,
        output: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        let mut response = message.into_response();
        match response.body.kind {
            UniqueIdBody::Generate => {
                response.body.kind = UniqueIdBody::GenerateOk {
                    id: Ulid::new().to_string(),
                }
            }
            UniqueIdBody::GenerateOk { .. } => {}
        };

        serde_json::to_writer(&mut *output, &response).context("failed to serialize echo")?;
        output.write_all(b"\n")?;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    start_node::<UniqueIdNode, UniqueIdBody>()?;
    Ok(())
}
