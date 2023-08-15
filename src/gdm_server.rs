use std::collections::HashMap;
use std::net::SocketAddr;

use anyhow::anyhow;
use bytebuffer::{ByteBuffer, ByteReader};
use log::{debug, error, info, warn};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

#[derive(Debug)]
enum Prefixes {
    Hello = 0x3,
    Ping = 0x0,
    Message = 0x1,
    Disconnect = 0x2,
    AckHello = 0x4,
    ServerData = 0x5,
    PlayerDisconnect = 0x7,
    PlayerIcons = 0x8,
    ReceivedPlayerIcons = 0x9,
    OutsideLevel = 0x10,
    VipActions = 0x11,
    BadKey = 0x12,
}

impl Prefixes {
    fn from_number(value: i8) -> Option<Self> {
        match value {
            0 => Some(Prefixes::Ping),
            1 => Some(Prefixes::Message),
            2 => Some(Prefixes::Disconnect),
            3 => Some(Prefixes::Hello),
            4 => Some(Prefixes::AckHello),
            5 => Some(Prefixes::ServerData),
            7 => Some(Prefixes::PlayerDisconnect),
            8 => Some(Prefixes::PlayerIcons),
            0x9 => Some(Prefixes::ReceivedPlayerIcons),
            0x10 => Some(Prefixes::OutsideLevel),
            0x11 => Some(Prefixes::VipActions),
            0x12 => Some(Prefixes::BadKey),
            _ => None,
        }
    }

    fn to_number(&self) -> i8 {
        match self {
            Prefixes::AckHello => 0x4,
            Prefixes::BadKey => 0x12,
            Prefixes::Disconnect => 0x2,
            Prefixes::VipActions => 0x11,
            Prefixes::OutsideLevel => 0x10,
            Prefixes::ReceivedPlayerIcons => 0x9,
            Prefixes::PlayerIcons => 0x8,
            Prefixes::PlayerDisconnect => 0x7,
            Prefixes::ServerData => 0x5,
            Prefixes::Hello => 0x3,
            Prefixes::Ping => 0x0,
            Prefixes::Message => 0x1,
        }
    }
}

#[derive(Debug)]
pub struct PlayerPosition {
    p1_pos: (i32, i32),
    p1_rot: (i32, i32),
    p1_gamemode: u8,
    p1_icon: u8,
    p1_size: i32,
    p1_gravity: u8,

    p2_pos: (i32, i32),
    p2_rot: (i32, i32),
    p2_gamemode: u8,
    p2_icon: u8,
    p2_size: i32,
    p2_gravity: u8,

    is_dead: u8,
    _room: i16,

    color1: u8,
    color2: u8,
    glow: u8,

    icon_ids: Vec<u8>,
}

pub struct State {
    levels: HashMap<i32, HashMap<i32, PlayerPosition>>,
    server_socket: Arc<UdpSocket>,
    client_to_addr: HashMap<i32, SocketAddr>,
    user_keys: HashMap<i32, u32>,
}

impl State {
    pub fn new(server_socket: Arc<UdpSocket>) -> Self {
        State {
            levels: HashMap::new(),
            server_socket,
            client_to_addr: HashMap::new(),
            user_keys: HashMap::new(),
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
            buf.write_i8(Prefixes::PlayerDisconnect.to_number());
            buf.write_i32(*client_left);
            self.send_to(client_id, buf.as_bytes()).await?;
        }
        Ok(())
    }

    pub async fn send_to(&self, client_id: &i32, data: &[u8]) -> anyhow::Result<usize> {
        let addr = self.client_to_addr.get(client_id);
        if addr.is_none() {
            return Err(anyhow!("Address not found"));
        }

        let addr = addr.unwrap();
        Ok(self.server_socket.send_to(data, addr).await?)
    }
}

pub async fn handle_packet(
    state: Arc<Mutex<State>>,
    buf: &[u8],
    address: SocketAddr,
) -> anyhow::Result<()> {
    let mut bytebuffer = ByteReader::from_bytes(buf);
    let prefix = Prefixes::from_number(bytebuffer.read_i8()?).ok_or(anyhow!("invalid prefix"))?;

    let client_id = bytebuffer.read_i32()?;
    let user_key = bytebuffer.read_u32()?;

    match prefix {
        Prefixes::Disconnect => {
            debug!("remote sent Prefixes::Disconnect");
            let mut state = state.lock().await;
            let clients = state.left_level(&client_id);
            state.notify_clients(&clients, &client_id).await?;
            state.client_to_addr.remove(&client_id);
            state.user_keys.remove(&client_id);
        }
        Prefixes::Hello => {
            debug!("remote sent Prefixes::Hello");
            let mut state = state.lock().await;
            state.client_to_addr.insert(client_id, address);
            state.user_keys.insert(client_id, user_key);

            let mut buf = ByteBuffer::new();
            buf.write_i8(Prefixes::AckHello.to_number());
            state.send_to(&client_id, buf.as_bytes()).await?;
        }
        Prefixes::Ping => {
            let res = bytebuffer.read_bytes(20);
            if res.is_ok() {
                let mut buf = ByteBuffer::new();
                buf.write_i8(Prefixes::Ping.to_number());
                buf.write_i32(0i32); // Online player count, unused
                let state = state.lock().await;
                state.send_to(&client_id, buf.as_bytes()).await?;
            } else {
                debug!("unhandled, got GetIcon or CreateLobby");
                // this never happens..?
                // handle GetIcon
                // CreateLobby can be passed too, but unimplemented
                // GetIcon -> 4 bytes - client ID
                // CreateLobby -> 1 byte - Prefixes::VipActions, 4 bytes - vipkey, u16 - lobby code
            }
        }
        Prefixes::OutsideLevel => {
            debug!("remote sent Prefixes::OutsideLevel");
            let mut state = state.lock().await;
            let correct_key = state.user_keys.get(&client_id).unwrap_or(&0u32);
            if *correct_key != user_key {
                warn!(
                    "client {client_id} wrong key, expected {correct_key}, client sent {user_key}"
                );
                let mut buf = ByteBuffer::new();
                buf.write_i8(Prefixes::Disconnect.to_number());
                state.send_to(&client_id, buf.as_bytes()).await?;
                return Ok(());
            }

            let clients = state.left_level(&client_id);
            state.notify_clients(&clients, &client_id).await?;
        }
        Prefixes::Message => {
            let mut state = state.lock().await;
            let correct_key = state.user_keys.get(&client_id).unwrap_or(&0u32);
            if *correct_key != user_key {
                warn!(
                    "client {client_id} wrong key, expected {correct_key}, client sent {user_key}"
                );
                let mut buf = ByteBuffer::new();
                buf.write_i8(Prefixes::Disconnect.to_number());
                state.send_to(&client_id, buf.as_bytes()).await?;
                return Ok(());
            }

            let p1_xpos = bytebuffer.read_i32()?;
            let p1_ypos = bytebuffer.read_i32()?;
            let p1_xrot = bytebuffer.read_i32()?;
            let p1_yrot = bytebuffer.read_i32()?;
            let p1_gamemode = bytebuffer.read_u8()?;
            let p1_active_icon_id = bytebuffer.read_u8()?;
            let p1_size = bytebuffer.read_i32()?;
            let p1_gravity = bytebuffer.read_u8()?;

            let p2_xpos = bytebuffer.read_i32()?;
            let p2_ypos = bytebuffer.read_i32()?;
            let p2_xrot = bytebuffer.read_i32()?;
            let p2_yrot = bytebuffer.read_i32()?;
            let p2_gamemode = bytebuffer.read_u8()?;
            let p2_active_icon_id = bytebuffer.read_u8()?;
            let p2_size = bytebuffer.read_i32()?;
            let p2_gravity = bytebuffer.read_u8()?;

            let is_dead = bytebuffer.read_u8()?;
            let level_id = bytebuffer.read_i32()?;
            let room = bytebuffer.read_i16()?;

            let color1 = bytebuffer.read_u8()?;
            let color2 = bytebuffer.read_u8()?;
            let glow = bytebuffer.read_u8()?;

            let icon_ids = bytebuffer.read_bytes(7)?;

            if level_id == -1 {
                let clients = state.left_level(&client_id);
                state.notify_clients(&clients, &client_id).await?;
            } else {
                let level = state.levels.entry(level_id).or_insert_with(HashMap::new);

                let pos_entry = PlayerPosition {
                    p1_pos: (p1_xpos, p1_ypos),
                    p1_rot: (p1_xrot, p1_yrot),
                    p1_gamemode,
                    p1_icon: p1_active_icon_id,
                    p1_size,
                    p1_gravity,

                    p2_pos: (p2_xpos, p2_ypos),
                    p2_rot: (p2_xrot, p2_yrot),
                    p2_gamemode,
                    p2_icon: p2_active_icon_id,
                    p2_size,
                    p2_gravity,

                    is_dead,
                    _room: room,

                    color1,
                    color2,
                    glow,

                    icon_ids,
                };

                if cfg!(debug_assertions) && !level.contains_key(&client_id) {
                    debug!("{client_id} join the level {level_id}");
                }

                level.insert(client_id, pos_entry);

                // get all players on the same level

                let players = state.levels.get(&level_id).unwrap();

                for (player_id, pos) in players.iter() {
                    if *player_id == client_id {
                        continue;
                    }

                    let mut buf = ByteBuffer::new();
                    buf.write_i8(Prefixes::Message.to_number());

                    buf.write_i32(*player_id);
                    buf.write_i32(pos.p1_pos.0);
                    buf.write_i32(pos.p1_pos.1);
                    buf.write_i32(pos.p1_rot.0);
                    buf.write_i32(pos.p1_rot.1);
                    buf.write_u8(pos.p1_gamemode);
                    buf.write_u8(pos.p1_icon);
                    buf.write_i32(pos.p1_size);
                    buf.write_u8(pos.p1_gravity);

                    buf.write_i32(pos.p2_pos.0);
                    buf.write_i32(pos.p2_pos.1);
                    buf.write_i32(pos.p2_rot.0);
                    buf.write_i32(pos.p2_rot.1);
                    buf.write_u8(pos.p2_gamemode);
                    buf.write_u8(pos.p2_icon);
                    buf.write_i32(pos.p2_size);
                    buf.write_u8(pos.p2_gravity);

                    buf.write_u8(pos.is_dead);
                    buf.write_u8(0u8); // ActiveIconId - unknown, unused?
                    buf.write_u8(pos.color1);
                    buf.write_u8(pos.color2);
                    buf.write_u8(pos.glow);
                    buf.write_bytes(&pos.icon_ids);

                    state.send_to(&client_id, buf.as_bytes()).await?;
                }
            }
        }
        _ => {
            warn!("invalid packet: {prefix:?}");
        }
    }

    Ok(())
}

pub async fn gdm_server(addr: &str, port: &str) -> anyhow::Result<()> {
    let addr = format!("{addr}:{port}");
    let socket = Arc::new(UdpSocket::bind(&addr).await?);

    let state = Arc::new(Mutex::new(State::new(socket.clone())));

    info!("GDM (UDP) server listening on: {addr}");

    let mut buf = [0u8; 4096];

    loop {
        let (len, peer) = socket.recv_from(&mut buf).await?;
        let cloned_state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_packet(cloned_state, &buf[..len], peer).await {
                error!("remote err from {peer}: {e}");
            }
        });
    }
}
