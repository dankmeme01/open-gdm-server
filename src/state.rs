use std::{collections::HashMap, sync::Arc, net::SocketAddr, time::{SystemTime, Duration}};

use anyhow::anyhow;
use bytebuffer::{ByteBuffer, Endian};
use log::debug;
use tokio::{net::UdpSocket, sync::Mutex};
use crate::gdm_server::{PlayerPosition, Prefixes};

pub struct State {
    pub levels: HashMap<i32, HashMap<i32, PlayerPosition>>,
    pub server_socket: Arc<UdpSocket>,
    pub connected_clients: HashMap<i32, (SocketAddr, u32, SystemTime)>, // client_id : address, user key, timestamp of last ping
}

impl State {
    pub fn new(server_socket: Arc<UdpSocket>) -> Self {
        State {
            levels: HashMap::new(),
            server_socket,
            connected_clients: HashMap::new(),
        }
    }

    pub fn left_level(&mut self, user: &i32) -> Vec<i32> {
        // returns users to notify about exit

        let mut users_in_level = vec![];
        // remove from existing levels, if applicable
        for (level_id, level_players) in self.levels.iter_mut() {
            if level_players.contains_key(user) {
                level_players.remove(user);
                users_in_level.extend(level_players.keys().copied().collect::<Vec<i32>>());
                debug!("{user} left the level {level_id}");
                break;
            }
        }

        // remove levels with 0 players
        self.levels.retain(|_, v| !v.is_empty());

        users_in_level
    }

    pub async fn notify_clients(
        &self,
        clients: &Vec<i32>,
        client_left: &i32,
    ) -> anyhow::Result<()> {
        debug!(
            "notifying {} clients that {client_left} left",
            clients.len()
        );
        for client_id in clients.iter() {
            let mut buf = ByteBuffer::new();
            buf.set_endian(Endian::LittleEndian);
            buf.write_i8(Prefixes::PlayerDisconnect.to_number());
            buf.write_i32(*client_left);
            self.send_to(client_id, buf.as_bytes()).await?;
        }
        Ok(())
    }

    pub async fn send_to(&self, client_id: &i32, data: &[u8]) -> anyhow::Result<usize> {
        let client = self.connected_clients.get(client_id);
        if client.is_none() {
            return Err(anyhow!("Client not found by id {client_id}"));
        }

        let client = client.unwrap();
        Ok(self.server_socket.send_to(data, client.0).await?)
    }

    pub async fn remove_dead_clients(&mut self) {
        let now = SystemTime::now();
        self.connected_clients.retain(|_, client| {
            let elapsed = now
                .duration_since(client.2)
                .unwrap_or_else(|_| Duration::from_secs(0));
            elapsed < Duration::from_secs(60)
        });
    }

    pub async fn update_client_time(&mut self, client_id: &i32) {
        if let Some(client) = self.connected_clients.get_mut(client_id) {
            client.2 = SystemTime::now();
        }
    }
}


// Thread Safe State shorthand
pub type TSState = Arc<Mutex<State>>;