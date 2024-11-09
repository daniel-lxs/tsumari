use std::net::ToSocketAddrs;
use std::sync::Arc;

use anyhow::Result;
use thrussh::client;
use thrussh::{ChannelMsg, Disconnect};

use thrussh_keys::load_secret_key;
use crate::models::Client;


pub struct Session {
  session: client::Handle<Client>,
  channel: Option<client::Channel>, // Store the shell channel
}

impl Session {
  pub async fn connect<A: ToSocketAddrs>(user: String, addrs: A) -> Result<Self> {
      let config = Arc::new(client::Config::default());
      let client_handler = Client {};

      let mut session = client::connect(config, addrs, client_handler).await?;

      let key_pair =
          match load_secret_key("/home/daniel/Workspace/tsumari/src-tauri/privatekey", None) {
              Ok(key) => key,
              Err(e) => {
                  println!("Error loading private key: {}", e);
                  return Err(e.into());
              }
          };

      session
          .authenticate_publickey(user, Arc::new(key_pair))
          .await?;

      // Request an interactive shell once
      let mut channel = session.channel_open_session().await?;
      channel.request_shell(true).await?;

      //Send a test command and wait for output to see if we are ready to process commands
      channel.data(&b"echo shell ready\n"[..]).await?;
      let tries = 10;
      let mut i = 0;
      loop {
          i += 1;
          if i == tries {
              break;
          }
          match channel.wait().await {
              Some(ChannelMsg::Data { data }) => {
                  let output_data = String::from_utf8_lossy(&data);
                  if output_data.contains("shell ready") {
                      break;
                  }
              }
              _ => {}
          }
      }

      Ok(Self {
          session,
          channel: Some(channel),
      })
  }

  pub async fn send_command(&mut self, command: &str) -> Result<String> {
      let channel = self.channel.as_mut().expect("Shell channel not open");

      //Add a newline to the command
      channel.data(format!("{}\n", command).as_bytes()).await?;

      let mut output = String::new();

      loop {
          match channel.wait().await {
              Some(ChannelMsg::Data { data }) => {
                  let output_data = String::from_utf8_lossy(&data);
                  output.push_str(&output_data);
                  break;
              }
              Some(ChannelMsg::ExitStatus { exit_status }) => {
                  println!("Command exited with status: {}", exit_status);
                  break;
              }
              Some(ChannelMsg::Close) | Some(ChannelMsg::Eof) => {
                  println!("Channel closed unexpectedly.");
                  break;
              }
              _ => {}
          }
      }
      Ok(output.trim_end_matches('\n').to_string())
  }

  pub async fn close(&mut self) -> Result<()> {
      self.session
          .disconnect(Disconnect::ByApplication, "", "English")
          .await?;
      Ok(())
  }
}