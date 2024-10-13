use std::io::{BufRead, Write};
use std::time::Duration;

use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<BodyKind> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<BodyKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Kind> {
    #[serde(flatten)]
    pub kind: Kind,
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
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
    fn new(id: String, node_ids: Vec<String>) -> Self;
    fn handle(
        &mut self,
        message: Message<NodeMessage>,
        output: &mut impl Write,
    ) -> anyhow::Result<()>;
    fn gossip(&mut self, output: &mut impl Write) -> anyhow::Result<()>;
}

enum Event<BodyKind> {
    NewMessage(Message<BodyKind>),
    Gossip,
}

pub fn start_node<N, NodeMessage>() -> anyhow::Result<()>
where
    N: Node<NodeMessage> + Send,
    NodeMessage: DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    let mut lines = std::io::stdin().lock().lines();
    let mut stdout = std::io::stdout().lock();

    let init_message = serde_json::from_str::<Message<InitBody>>(
        &lines.next().expect("no init message received")?,
    )?;
    let InitBody::Init { node_id, node_ids } = init_message.body.kind else {
        panic!("first message is not init");
    };

    let mut node: N = Node::new(node_id, node_ids);

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

    let (tx, rx) = std::sync::mpsc::channel();
    let message_tx = tx.clone();
    let gossip_tx = tx.clone();

    drop(lines);
    let jh = std::thread::spawn(move || {
        let lines = std::io::stdin().lock().lines();
        for line in lines {
            let message = serde_json::from_str::<Message<NodeMessage>>(
                &line.context("could not read from STDIN")?,
            )
            .context("could not deserialize from STDIN")?;
            if let Err(_) = message_tx.send(Event::NewMessage(message)) {
                return Ok::<_, anyhow::Error>(());
            }
        }

        Ok(())
    });

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(500));
            if let Err(_) = gossip_tx.send(Event::Gossip) {
                break;
            }
        }

        anyhow::Ok(())
    });

    for event in rx {
        match event {
            Event::NewMessage(message) => {
                node.handle(message, &mut stdout)?;
            }
            Event::Gossip => {
                node.gossip(&mut stdout)?;
            }
        }
    }
    jh.join()
        .expect("message thread panic")
        .context("message thread error")?;
    Ok(())
}
