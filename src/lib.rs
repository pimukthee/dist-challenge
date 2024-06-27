use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<BodyKind> {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body<BodyKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body<Kind> {
    #[serde(flatten)]
    kind: Kind,
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
}

pub trait IntoResponse {
    fn into_response(self) -> Self;
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

impl IntoResponse for InitBody {
    fn into_response(self) -> InitBody {
        match self {
            InitBody::Init { .. } => InitBody::InitOk,
            _ => unimplemented!(),
        }
    }
}

impl<BodyKind: IntoResponse + Serialize> Message<BodyKind> {
    pub fn into_response(self, output: &mut impl std::io::Write) -> anyhow::Result<()> {
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
