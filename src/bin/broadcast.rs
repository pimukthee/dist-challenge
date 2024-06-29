use anyhow::Context;
use dist_challenge::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastBody {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct BroadcastNode {
    id: String,
    neighbors: Vec<String>,
    messages: Vec<usize>,
}

impl Node<BroadcastBody> for BroadcastNode {
    fn new(id: String) -> Self {
        Self {
            id,
            neighbors: vec![],
            messages: vec![],
        }
    }

    fn handle(
        &mut self,
        message: Message<BroadcastBody>,
        output: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        let mut response = message.into_response();
        match response.body.kind {
            BroadcastBody::Broadcast { message } => {
                response.body.kind = BroadcastBody::BroadcastOk;
                self.messages.push(message);
            }
            BroadcastBody::Read => {
                response.body.kind = BroadcastBody::ReadOk {
                    messages: self.messages.clone(),
                };
            }
            BroadcastBody::Topology { mut topology } => {
                response.body.kind = BroadcastBody::TopologyOk;
                self.neighbors = topology.remove(&self.id).unwrap_or(vec![]);
            }

            BroadcastBody::TopologyOk
            | BroadcastBody::ReadOk { .. }
            | BroadcastBody::BroadcastOk => {}
        };

        serde_json::to_writer(&mut *output, &response).context("cannot serialize message")?;
        output.write(b"\n")?;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    start_node::<BroadcastNode, BroadcastBody>()?;

    Ok(())
}
