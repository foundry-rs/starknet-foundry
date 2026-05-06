use std::{
    borrow::Cow,
    io::{BufRead, BufReader},
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};

use reqwest::{Client, ClientBuilder};
use serde::{Serialize, ser::SerializeSeq};

#[derive(Debug)]
pub struct SpeculosClient {
    process: Child,
    port: u16,
    client: Client,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutomationRule<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regexp: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<u32>,
    pub conditions: &'a [AutomationCondition<'a>],
    pub actions: &'a [AutomationAction<'a>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationAction<'a> {
    Button { button: Button, pressed: bool },
    Setbool { varname: Cow<'a, str>, value: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationCondition<'a> {
    pub varname: Cow<'a, str>,
    pub value: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
}

#[derive(Debug)]
pub enum SpeculosError {
    IoError(std::io::Error),
    ReqwestError(reqwest::Error),
}

#[derive(Serialize)]
struct PostAutomationRequest<'a> {
    version: u32,
    rules: &'a [AutomationRule<'a>],
}

impl SpeculosClient {
    pub fn new<P: AsRef<Path>>(port: u16, app: P) -> Result<Self, SpeculosError> {
        Self::new_with_timeout(port, app, Duration::from_secs(10))
    }

    pub fn new_with_timeout<P: AsRef<Path>>(
        port: u16,
        app: P,
        timeout: Duration,
    ) -> Result<Self, SpeculosError> {
        let mut process = Command::new("speculos")
            .args([
                "--api-port",
                &port.to_string(),
                "--apdu-port",
                "0",
                "-m",
                "nanox",
                "--display",
                "headless",
                &app.as_ref().display().to_string(),
            ])
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(stderr) = process.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                if line.contains("launcher: using default app name & version")
                    || line.contains("[*] Env app version:")
                {
                    break;
                }
            }
        }

        let addr = format!("127.0.0.1:{port}");
        let deadline = std::time::Instant::now() + timeout;
        'poll: while std::time::Instant::now() < deadline {
            if let Ok(mut stream) = std::net::TcpStream::connect(&addr) {
                use std::io::{Read, Write};
                let req = format!("GET /events HTTP/1.0\r\nHost: localhost:{port}\r\n\r\n");
                if stream.write_all(req.as_bytes()).is_ok() {
                    let mut resp = String::new();
                    let _ = stream.read_to_string(&mut resp);
                    if resp.contains("\"text\"") {
                        break 'poll;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(Self {
            process,
            port,
            client: ClientBuilder::new().timeout(timeout).build().unwrap(),
        })
    }

    pub async fn automation(&self, rules: &[AutomationRule<'_>]) -> Result<(), SpeculosError> {
        let response = self
            .client
            .post(format!("http://localhost:{}/automation", self.port))
            .json(&PostAutomationRequest { version: 1, rules })
            .send()
            .await?;
        response.error_for_status()?;
        Ok(())
    }

    pub async fn click_button(&self, button: Button) -> Result<(), SpeculosError> {
        #[derive(serde::Serialize)]
        struct ButtonRequest {
            action: &'static str,
        }
        let name = match button {
            Button::Left => "left",
            Button::Right => "right",
        };
        let response = self
            .client
            .post(format!("http://localhost:{}/button/{name}", self.port))
            .json(&ButtonRequest {
                action: "press-and-release",
            })
            .send()
            .await?;
        response.error_for_status()?;
        Ok(())
    }
}

impl Drop for SpeculosClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

impl Serialize for AutomationCondition<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.varname)?;
        seq.serialize_element(&self.value)?;
        seq.end()
    }
}

impl Serialize for AutomationAction<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Button { button, pressed } => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("button")?;
                seq.serialize_element(&match button {
                    Button::Left => 1,
                    Button::Right => 2,
                })?;
                seq.serialize_element(pressed)?;
                seq.end()
            }
            Self::Setbool { varname, value } => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("setbool")?;
                seq.serialize_element(varname)?;
                seq.serialize_element(value)?;
                seq.end()
            }
        }
    }
}

impl From<std::io::Error> for SpeculosError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<reqwest::Error> for SpeculosError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

impl std::fmt::Display for SpeculosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "{e}"),
            Self::ReqwestError(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SpeculosError {}
