use crate::interactivity::SafeDespawnExt;
use crate::net::actor::RemotePlayerActor;
use crate::net::protocol::NetPacket;
use crate::net::scoreboard::DukematchState;
use crate::player::types::{PlayerController, PlayerId};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub player_id: usize,
    pub addr: SocketAddr,
    pub name: String,
    pub color_pal: u8,
    pub last_seen_sec: f32,
    pub ping_ms: u32,
}

#[derive(Resource)]
pub struct NetTransport {
    pub socket: Option<Arc<UdpSocket>>,
    pub local_addr: Option<SocketAddr>,
    pub peers: HashMap<SocketAddr, PeerInfo>,
    pub timeout_sec: f32,
    pub heartbeat_timer: f32,
    pub is_server: bool,
}

impl Default for NetTransport {
    fn default() -> Self {
        Self {
            socket: None,
            local_addr: None,
            peers: HashMap::new(),
            timeout_sec: 5.0,
            heartbeat_timer: 0.0,
            is_server: false,
        }
    }
}

impl NetTransport {
    pub fn bind(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        let local_addr = socket.local_addr()?;
        Ok(Self {
            socket: Some(Arc::new(socket)),
            local_addr: Some(local_addr),
            peers: HashMap::new(),
            timeout_sec: 5.0,
            heartbeat_timer: 0.0,
            is_server: false,
        })
    }

    pub fn bind_client() -> std::io::Result<Self> {
        Self::bind("0.0.0.0:0")
    }

    pub fn bind_server(port: u16) -> std::io::Result<Self> {
        let mut transport = Self::bind(&format!("0.0.0.0:{}", port))?;
        transport.is_server = true;
        Ok(transport)
    }

    pub fn send_packet(&self, packet: &NetPacket, addr: SocketAddr) -> std::io::Result<usize> {
        if let Some(sock) = &self.socket {
            let bytes = packet.to_bytes();
            sock.send_to(&bytes, addr)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Socket not bound",
            ))
        }
    }

    pub fn broadcast(&self, packet: &NetPacket) -> usize {
        let mut count = 0;
        for &addr in self.peers.keys() {
            if self.send_packet(packet, addr).is_ok() {
                count += 1;
            }
        }
        count
    }

    pub fn poll_packets(&mut self) -> Vec<(SocketAddr, NetPacket)> {
        let mut received = Vec::new();
        let sock = match &self.socket {
            Some(s) => s.clone(),
            None => return received,
        };

        let mut buf = [0u8; 2048];
        loop {
            match sock.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    if let Ok(pkt) = NetPacket::from_bytes(&buf[..len]) {
                        if let Some(peer) = self.peers.get_mut(&addr) {
                            peer.last_seen_sec = 0.0;
                        }
                        received.push((addr, pkt));
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                    break;
                }
                Err(_) => break,
            }
        }
        received
    }

    pub fn register_peer(
        &mut self,
        addr: SocketAddr,
        player_id: usize,
        name: String,
        color_pal: u8,
        current_time: f32,
    ) {
        self.peers.insert(
            addr,
            PeerInfo {
                player_id,
                addr,
                name,
                color_pal,
                last_seen_sec: current_time,
                ping_ms: 0,
            },
        );
    }

    pub fn disconnect_peer(&mut self, addr: &SocketAddr) -> Option<PeerInfo> {
        self.peers.remove(addr)
    }

    pub fn disconnect_peer_by_id(&mut self, player_id: usize) -> Option<PeerInfo> {
        let target_addr = self
            .peers
            .iter()
            .find(|(_, p)| p.player_id == player_id)
            .map(|(a, _)| *a);

        if let Some(addr) = target_addr {
            self.peers.remove(&addr)
        } else {
            None
        }
    }

    pub fn check_timeouts(&mut self, now_sec: f32) -> Vec<usize> {
        let mut timed_out = Vec::new();
        let timeout = self.timeout_sec;
        self.peers.retain(|_, peer| {
            if now_sec - peer.last_seen_sec > timeout {
                timed_out.push(peer.player_id);
                false
            } else {
                true
            }
        });
        timed_out
    }

    pub fn get_peer_by_player_id(&self, player_id: usize) -> Option<&PeerInfo> {
        self.peers.values().find(|p| p.player_id == player_id)
    }
}

/// Component for smoothly interpolating remote player transforms received over UDP.
#[derive(Component, Debug, Clone)]
pub struct RemotePlayerNetworkSync {
    pub player_id: usize,
    pub target_pos: Vec3,
    pub target_yaw: f32,
    pub target_pitch: f32,
    pub smoothing_factor: f32,
    pub last_gametic: u32,
}

impl Default for RemotePlayerNetworkSync {
    fn default() -> Self {
        Self {
            player_id: 0,
            target_pos: Vec3::ZERO,
            target_yaw: 0.0,
            target_pitch: 0.0,
            smoothing_factor: 15.0,
            last_gametic: 0,
        }
    }
}

/// Helper function to interpolate a transform towards a target position and yaw.
pub fn interpolate_transform(
    current_pos: Vec3,
    target_pos: Vec3,
    current_yaw: f32,
    target_yaw: f32,
    smoothing_factor: f32,
    dt: f32,
) -> (Vec3, f32) {
    let alpha = (smoothing_factor * dt).clamp(0.0, 1.0);
    let new_pos = current_pos.lerp(target_pos, alpha);

    let delta_yaw = (target_yaw - current_yaw + PI).rem_euclid(TAU) - PI;
    let new_yaw = current_yaw + delta_yaw * alpha;
    (new_pos, new_yaw)
}

/// System to smoothly interpolate remote player positions and rotations every frame.
pub fn update_remote_network_interpolation(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &RemotePlayerNetworkSync)>,
) {
    let dt = time.delta_seconds();
    for (mut transform, sync) in query.iter_mut() {
        let current_yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
        let (new_pos, new_yaw) = interpolate_transform(
            transform.translation,
            sync.target_pos,
            current_yaw,
            sync.target_yaw,
            sync.smoothing_factor,
            dt,
        );
        transform.translation = new_pos;
        transform.rotation = Quat::from_rotation_y(new_yaw);
    }
}

/// System to broadcast local player state (and heartbeats) over UDP.
pub fn net_send_sync_system(
    time: Res<Time>,
    mut transport: ResMut<NetTransport>,
    dukematch_state: Res<DukematchState>,
    player_query: Query<(&Transform, &PlayerController, &PlayerId)>,
) {
    if transport.socket.is_none() || transport.peers.is_empty() {
        return;
    }

    let local_id = dukematch_state.local_player_id as usize;

    // Send local player state sync
    for (transform, controller, id) in player_query.iter() {
        if id.0 == local_id {
            let pkt = NetPacket::PlayerStateSync {
                player_id: local_id as u8,
                x: transform.translation.x,
                y: transform.translation.y,
                z: transform.translation.z,
                yaw: controller.yaw,
                pitch: controller.pitch,
                health: controller.health as i16,
                armor: controller.armor as i16,
            };
            transport.broadcast(&pkt);
            break;
        }
    }

    // Periodic heartbeat every 1.0s
    transport.heartbeat_timer += time.delta_seconds();
    if transport.heartbeat_timer >= 1.0 {
        transport.heartbeat_timer = 0.0;
        let heartbeat = NetPacket::Heartbeat {
            player_id: local_id as u8,
        };
        transport.broadcast(&heartbeat);
    }
}

/// System to receive pending UDP packets and update game state/entities.
pub fn net_receive_packets_system(
    time: Res<Time>,
    mut commands: Commands,
    mut transport: ResMut<NetTransport>,
    mut dukematch_state: ResMut<DukematchState>,
    mut sync_query: Query<(Entity, &mut RemotePlayerNetworkSync, &mut RemotePlayerActor)>,
    player_query: Query<(Entity, &PlayerId)>,
) {
    if transport.socket.is_none() {
        return;
    }

    let current_time = time.elapsed_seconds();
    let packets = transport.poll_packets();

    for (addr, packet) in packets {
        // Update peer last_seen
        if let Some(peer) = transport.peers.get_mut(&addr) {
            peer.last_seen_sec = current_time;
        }

        match packet {
            NetPacket::Connect {
                player_id,
                name,
                color_pal,
            } => {
                let pid = player_id as usize;
                transport.register_peer(addr, pid, name.clone(), color_pal, current_time);

                if pid < 8 {
                    dukematch_state.player_names[pid] = name;
                    dukematch_state.player_colors[pid] = color_pal;
                }

                // Check if remote player entity exists, spawn if not
                let existing = player_query.iter().any(|(_, id)| id.0 == pid);
                if !existing && pid != dukematch_state.local_player_id as usize {
                    commands.spawn((
                        RemotePlayerActor {
                            player_id: pid,
                            palette_idx: color_pal,
                            ..default()
                        },
                        RemotePlayerNetworkSync {
                            player_id: pid,
                            smoothing_factor: 15.0,
                            ..default()
                        },
                        PlayerId(pid),
                        Transform::from_translation(Vec3::ZERO),
                        GlobalTransform::default(),
                        VisibilityBundle::default(),
                    ));
                }

                // If server, send back our own Connect info to complete handshake
                if transport.is_server {
                    let local_id = dukematch_state.local_player_id;
                    let reply = NetPacket::Connect {
                        player_id: local_id,
                        name: dukematch_state.player_names[local_id as usize].clone(),
                        color_pal: dukematch_state.player_colors[local_id as usize],
                    };
                    let _ = transport.send_packet(&reply, addr);
                }
            }
            NetPacket::Disconnect { player_id } => {
                let pid = player_id as usize;
                transport.disconnect_peer_by_id(pid);
                for (entity, id) in player_query.iter() {
                    if id.0 == pid {
                        commands.safe_despawn_recursive(entity);
                    }
                }
            }
            NetPacket::Heartbeat { .. } => {
                // Already updated peer.last_seen_sec above
            }
            NetPacket::PlayerStateSync {
                player_id,
                x,
                y,
                z,
                yaw,
                pitch,
                ..
            } => {
                let pid = player_id as usize;
                for (_, mut sync, mut actor) in sync_query.iter_mut() {
                    if sync.player_id == pid {
                        sync.target_pos = Vec3::new(x, y, z);
                        sync.target_yaw = yaw;
                        sync.target_pitch = pitch;
                        actor.yaw = yaw;
                        break;
                    }
                }
            }
            NetPacket::InputSync { .. } => {
                // Input lockstep / prediction support
            }
            NetPacket::FragEvent {
                killer_id,
                victim_id,
                ..
            } => {
                dukematch_state.record_frag(killer_id as usize, victim_id as usize);
            }
            NetPacket::ChatMessage { .. } => {
                // Broadcast / chat support
            }
        }
    }
}

/// System to detect and disconnect timed-out peers.
pub fn net_peer_timeout_system(
    time: Res<Time>,
    mut commands: Commands,
    mut transport: ResMut<NetTransport>,
    player_query: Query<(Entity, &PlayerId)>,
) {
    if transport.socket.is_none() {
        return;
    }

    let timed_out_ids = transport.check_timeouts(time.elapsed_seconds());
    for pid in timed_out_ids {
        for (entity, id) in player_query.iter() {
            if id.0 == pid {
                commands.safe_despawn_recursive(entity);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_socket_initialization_nonblocking() {
        let server = NetTransport::bind("127.0.0.1:0").expect("Failed to bind server socket");
        assert!(server.socket.is_some());
        assert!(server.local_addr.is_some());
        assert_eq!(server.peers.len(), 0);

        let mut client = NetTransport::bind("127.0.0.1:0").expect("Failed to bind client socket");
        assert!(client.socket.is_some());
        assert!(client.local_addr.is_some());

        // Polling when no packets are present should return immediately (non-blocking)
        let packets = client.poll_packets();
        assert!(packets.is_empty());
    }

    #[test]
    fn test_peer_handshake_and_loopback_packet_transmission() {
        let mut server = NetTransport::bind("127.0.0.1:0").expect("Failed to bind server socket");
        server.is_server = true;
        let server_addr = server.local_addr.unwrap();

        let mut client = NetTransport::bind("127.0.0.1:0").expect("Failed to bind client socket");
        let client_addr = client.local_addr.unwrap();

        // Client connects to Server
        let connect_pkt = NetPacket::Connect {
            player_id: 1,
            name: "Caleb".to_string(),
            color_pal: 10,
        };
        client
            .send_packet(&connect_pkt, server_addr)
            .expect("Failed to send connect packet");

        // Small spin or direct poll to receive packet over loopback
        let mut received = Vec::new();
        for _ in 0..20 {
            received = server.poll_packets();
            if !received.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        assert_eq!(received.len(), 1);
        let (from_addr, pkt) = &received[0];
        assert_eq!(*from_addr, client_addr);
        assert_eq!(*pkt, connect_pkt);

        // Server registers client
        server.register_peer(client_addr, 1, "Caleb".to_string(), 10, 0.0);
        assert_eq!(server.peers.len(), 1);

        // Server sends PlayerStateSync to Client
        let state_pkt = NetPacket::PlayerStateSync {
            player_id: 0,
            x: 100.0,
            y: 0.0,
            z: -50.0,
            yaw: 1.57,
            pitch: 0.0,
            health: 100,
            armor: 50,
        };
        server
            .send_packet(&state_pkt, client_addr)
            .expect("Failed to send state sync");

        let mut client_received = Vec::new();
        for _ in 0..20 {
            client_received = client.poll_packets();
            if !client_received.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        assert_eq!(client_received.len(), 1);
        assert_eq!(client_received[0].1, state_pkt);

        // Client sends disconnect
        let disc_pkt = NetPacket::Disconnect { player_id: 1 };
        client.send_packet(&disc_pkt, server_addr).unwrap();

        for _ in 0..20 {
            let packets = server.poll_packets();
            if !packets.is_empty() {
                assert_eq!(packets[0].1, disc_pkt);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        server.disconnect_peer(&client_addr);
        assert_eq!(server.peers.len(), 0);
    }

    #[test]
    fn test_remote_network_interpolation_and_smoothing() {
        let current_pos = Vec3::new(0.0, 0.0, 0.0);
        let target_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_yaw = 0.0;
        let target_yaw = 1.0;

        let (new_pos, new_yaw) =
            interpolate_transform(current_pos, target_pos, current_yaw, target_yaw, 10.0, 0.05);

        // Alpha = 10.0 * 0.05 = 0.5
        assert!((new_pos.x - 5.0).abs() < 1e-4);
        assert!((new_yaw - 0.5).abs() < 1e-4);

        // Test shortest-path angular delta wrapping
        let current_yaw_wrap = 3.0; // close to PI
        let target_yaw_wrap = -3.0; // close to -PI
        let (_, wrapped_yaw) = interpolate_transform(
            current_pos,
            target_pos,
            current_yaw_wrap,
            target_yaw_wrap,
            10.0,
            0.05,
        );
        // Shortest turn crosses PI / -PI boundary rather than rotating 6 radians backwards
        assert!(wrapped_yaw > 3.0 || wrapped_yaw < -3.0);
    }

    #[test]
    fn test_peer_timeout_detection() {
        let mut transport = NetTransport::default();
        let addr1: SocketAddr = "127.0.0.1:8001".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:8002".parse().unwrap();

        transport.register_peer(addr1, 1, "Player 1".to_string(), 0, 10.0);
        transport.register_peer(addr2, 2, "Player 2".to_string(), 9, 13.0);

        // At t = 14.0 (elapsed since last_seen: p1=4.0s, p2=1.0s, timeout=5.0s)
        let timed_out = transport.check_timeouts(14.0);
        assert!(timed_out.is_empty());
        assert_eq!(transport.peers.len(), 2);

        // At t = 16.0 (elapsed since last_seen: p1=6.0s > 5.0s, p2=3.0s < 5.0s)
        let timed_out = transport.check_timeouts(16.0);
        assert_eq!(timed_out, vec![1]);
        assert_eq!(transport.peers.len(), 1);
        assert!(transport.get_peer_by_player_id(2).is_some());
        assert!(transport.get_peer_by_player_id(1).is_none());
    }

    #[test]
    fn test_broadcast_to_multiple_peers() {
        let mut server = NetTransport::bind("127.0.0.1:0").expect("Failed to bind server");
        let client1 = NetTransport::bind("127.0.0.1:0").expect("Failed to bind client 1");
        let client2 = NetTransport::bind("127.0.0.1:0").expect("Failed to bind client 2");

        let addr1 = client1.local_addr.unwrap();
        let addr2 = client2.local_addr.unwrap();

        server.register_peer(addr1, 1, "P1".into(), 0, 0.0);
        server.register_peer(addr2, 2, "P2".into(), 9, 0.0);

        let chat = NetPacket::ChatMessage {
            sender_id: 0,
            message: "Groovy".into(),
        };
        let sent_count = server.broadcast(&chat);
        assert_eq!(sent_count, 2);
    }
}
