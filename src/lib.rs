use std::io::{BufRead, Write};

use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<BodyKind> {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    pub body: Body<BodyKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Kind> {
    #[serde(flatten)]
    pub kind: Kind,
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitBody {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

impl<BodyKind> Message<BodyKind> {
    pub fn into_response(self) -> Message<BodyKind> {
        Message {
            src: self.dst,
            dst: self.src,
            body: Body {
                msg_id: Some(0),
                in_reply_to: self.body.msg_id,
                kind: self.body.kind,
            },
        }
    }
}

pub trait Node<NodeMessage> {
    fn new(id: String) -> Self;
    fn handle(
        &mut self,
        message: Message<NodeMessage>,
        output: &mut impl Write,
    ) -> anyhow::Result<()>;
}

pub fn start_node<N: Node<NodeMessage>, NodeMessage: DeserializeOwned>() -> anyhow::Result<()> {
    let mut lines = std::io::stdin().lock().lines();
    let mut stdout = std::io::stdout().lock();

    let init_message = serde_json::from_str::<Message<InitBody>>(
        &lines.next().expect("no init message received")?,
    )?;
    let InitBody::Init { node_id, .. } = init_message.body.kind else {
        panic!("first message is not init");
    };

    let mut node: N = Node::new(node_id);

    let response = Message {
        src: init_message.dst,
        dst: init_message.src,
        body: Body {
            msg_id: Some(0),
            in_reply_to: init_message.body.msg_id,
            kind: InitBody::InitOk,
        },
    };
    serde_json::to_writer(&mut stdout, &response).context("failed to parse init ok")?;
    stdout.write_all(b"\n")?;

    for line in lines {
        let message = serde_json::from_str::<Message<NodeMessage>>(
            &line.context("could not read from STDIN")?,
        )
        .context("could not deserialize from STDIN")?;

        node.handle(message, &mut stdout)?;
    }

    Ok(())
}
