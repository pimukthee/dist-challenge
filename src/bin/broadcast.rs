use anyhow::Context;
use dist_challenge::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;

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
    Gossip {
        messages: Vec<usize>,
    },
    GossipOk {
        messages: Vec<usize>,
    },
}

struct BroadcastNode {
    id: String,
    neighbors: Vec<String>,
    messages: Vec<usize>,
    seen: HashMap<String, HashSet<usize>>,
}

impl Node<BroadcastBody> for BroadcastNode {
    fn new(id: String, node_ids: Vec<String>) -> Self {
        Self {
            id,
            neighbors: vec![],
            messages: vec![],
            seen: node_ids
                .into_iter()
                .map(|id| (id, HashSet::new()))
                .collect(),
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
            BroadcastBody::Gossip { messages } => {
                self.seen
                    .get_mut(&response.dst)
                    .unwrap()
                    .extend(messages.iter().copied());
                self.messages.extend(messages.iter());
                response.body.kind = BroadcastBody::GossipOk { messages };
            }
            BroadcastBody::GossipOk { messages } => {
                self.seen
                    .get_mut(&response.dst)
                    .unwrap()
                    .extend(messages.iter().copied());
                return Ok(());
            }

            BroadcastBody::TopologyOk
            | BroadcastBody::ReadOk { .. }
            | BroadcastBody::BroadcastOk => {}
        };

        serde_json::to_writer(&mut *output, &response).context("cannot serialize message")?;
        output.write(b"\n")?;

        Ok(())
    }

    fn gossip(&mut self, output: &mut impl Write) -> anyhow::Result<()> {
        for n in &self.neighbors {
            let messages = self
                .messages
                .iter()
                .copied()
                .filter(|&message| !self.seen.get(n).unwrap().contains(&message))
                .collect::<Vec<_>>();

            if messages.is_empty() {
                continue;
            }

            let message = Message {
                src: self.id.clone(),
                dst: n.to_string(),
                body: Body {
                    msg_id: None,
                    in_reply_to: None,
                    kind: BroadcastBody::Gossip { messages },
                },
            };
            serde_json::to_writer(&mut *output, &message).context("cannot serialize message")?;
            output.write(b"\n")?;
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    start_node::<BroadcastNode, BroadcastBody>()?;

    Ok(())
}
