pub mod commands;

use commands::{Command, AppEvent};
use tokio::sync::{mpsc, broadcast};
use anyhow::Result;

use crate::storage::Database;
use crate::p2p::{NetworkManager, ChatTicket};
use crate::crypto::{CryptoManager, Identity};

pub struct NodeChatWorker {
    db_path: String,
    network: Option<NetworkManager>,
    crypto: CryptoManager,
    rx_commands: mpsc::Receiver<Command>,
    tx_events: broadcast::Sender<AppEvent>,
}

impl NodeChatWorker {
    pub fn new(
        db_path: String,
        rx_commands: mpsc::Receiver<Command>,
        tx_events: broadcast::Sender<AppEvent>,
    ) -> Self {
        Self {
            db_path,
            network: None,
            crypto: CryptoManager::new(),
            rx_commands,
            tx_events,
        }
    }

    /// Primary asynchronous routine executing networking, crypto, and DB commands.
    pub async fn run(mut self) -> Result<()> {
        println!("[Worker] Secure Backend Initializing...");
        
        let tx = self.tx_events.clone();
        
        // 1. Initialize the Database in the background
        let db = match Database::new(&self.db_path) {
            Ok(d) => d,
            Err(e) => {
                let _ = tx.send(AppEvent::ErrorMessage(format!("Database failed to open: {}", e)));
                return Err(e);
            }
        };

        // 2. Initialize networking and identity in background
        let secret_key = match db.get_or_create_iroh_key() {
            Ok(key) => key,
            Err(e) => {
                let msg = format!("Failed to access or generate Identity DB: {}", e);
                eprintln!("[Worker Error] {}", msg);
                let _ = tx.send(AppEvent::ErrorMessage(msg));
                return Err(e);
            }
        };

        println!("[Worker] Bootstrapping Iroh Identity...");
        let tx_net = self.tx_events.clone();
        let seed = secret_key.to_bytes();
        let identity = Identity::from_seed(&seed);
        let x_public = identity.x25519_public.to_bytes();
        
        self.crypto.set_identity(identity);
        
        match NetworkManager::new(secret_key, tx_net.clone()).await {
            Ok(net) => {
                let node_id = net.node_id().to_string();
                let display_name = db.get_config("display_name").ok()
                    .flatten()
                    .map(|b| String::from_utf8_lossy(&b).into_owned());
                
                if let Ok(ticket) = net.create_ticket(x_public).await {
                    let _ = tx_net.send(AppEvent::IdentityCreated { 
                        node_id, 
                        ticket: ticket.to_string(),
                        display_name,
                    });
                }
                self.network = Some(net);
            }
            Err(e) => {
                let msg = format!("Network init failed. Is port blocked? {}", e);
                eprintln!("[Worker Error] {}", msg);
                let _ = tx_net.send(AppEvent::ErrorMessage(msg));
            }
        }
        
        let _ = tx.send(AppEvent::BackendReady);
        let mut rx_event = self.tx_events.subscribe();

        loop {
            tokio::select! {
                // 1. Process Frontend Commands
                cmd = self.rx_commands.recv() => {
                    match cmd {
                        Some(Command::Quit) => {
                            println!("[Worker] Graceful shutdown initiated.");
                            break Ok(());
                        }
                        Some(command) => {
                            if let Err(e) = self.handle_command(&db, command).await {
                                eprintln!("[Worker] Command handling error: {}", e);
                            }
                        }
                        None => {
                            println!("[Worker] Command channel closed. Shutting down.");
                            break Ok(());
                        }
                    }
                }

                // 3. Process Backend Events (Internal decryption & routing)
                Ok(event) = rx_event.recv() => {
                    match event {
                        AppEvent::InternalIncomingMessage { sender, ciphertext } => {
                            if let (Some(identity), Ok(_)) = (&self.crypto.identity, sender.parse::<iroh::EndpointId>()) {
                                if let Ok(Some(peer_pk_bytes)) = db.get_peer_key(&sender) {
                                    let peer_pk = x25519_dalek::PublicKey::from(<[u8; 32]>::try_from(peer_pk_bytes).unwrap());
                                    let shared_secret = CryptoManager::derive_shared_secret(&identity.x25519_secret, &peer_pk);
                                    
                                    if let Ok(plaintext_bytes) = CryptoManager::decrypt_direct(&ciphertext, &shared_secret) {
                                        let plaintext = String::from_utf8_lossy(&plaintext_bytes).to_string();
                                        let _ = self.tx_events.send(AppEvent::IncomingMessage { sender, plaintext });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Processes high-level actions sent from the UI.
    async fn handle_command(&mut self, db: &Database, cmd: Command) -> Result<()> {
        match cmd {
            Command::UpdateProfile { display_name } => {
                println!("[Worker] Setting display name: {}", display_name);
                db.set_config("display_name", display_name.as_bytes())?;

                if self.network.is_none() {
                    let secret_key = db.get_or_create_iroh_key()?;
                    let seed = secret_key.to_bytes();
                    
                    let identity = Identity::from_seed(&seed);
                    let x_public = identity.x25519_public.to_bytes();
                    self.crypto.set_identity(identity);

                    let network = NetworkManager::new(secret_key, self.tx_events.clone()).await?;
                    let node_id = network.node_id().to_string();
                    
                    // Create the shareable ticket
                    let ticket = network.create_ticket(x_public).await?.to_string();
                    self.network = Some(network);
                    
                    let _ = self.tx_events.send(AppEvent::IdentityCreated { 
                        node_id, 
                        ticket,
                        display_name: Some(display_name),
                    });
                }
            }
            Command::RefreshIdentity => {
               if let (Some(net), Some(identity)) = (&self.network, &self.crypto.identity) {
                   let node_id = net.node_id().to_string();
                   let display_name = db.get_config("display_name").ok().and_then(|o| o.map(|b| String::from_utf8_lossy(&b).to_string()));
                   let x_public = identity.x25519_public.to_bytes();
                   if let Ok(ticket) = net.create_ticket(x_public).await {
                       let _ = self.tx_events.send(AppEvent::IdentityCreated { 
                           node_id, 
                           ticket: ticket.to_string(),
                           display_name,
                       });
                   }
               } else {
                   let _ = self.tx_events.send(AppEvent::ErrorMessage("P2P Network is not running. Node ID generated was empty!".into()));
               }
            }
            Command::AddContactByTicket { ticket: ticket_str } => {
                println!("[Worker] Adding contact from ticket...");
                match ticket_str.parse::<ChatTicket>() {
                    Ok(ticket) => {
                        let node_id = ticket.addr.id.to_string();
                        // Store the peer's X25519 public key
                        if let Err(e) = db.upsert_peer(&node_id, &ticket.x25519_public, None) {
                            let _ = self.tx_events.send(AppEvent::ErrorMessage(format!("Db error: {}", e)));
                        } else {
                            println!("[Worker] Contact added: {}", node_id);
                            // We can also trigger a first ping or discovery here
                        }
                    }
                    Err(e) => {
                        let _ = self.tx_events.send(AppEvent::ErrorMessage(format!("Invalid ticket: {}", e)));
                    }
                }
            }
            Command::SendDirectMessage { target, plaintext } => {
                if let (Some(net), Some(identity)) = (&self.network, &self.crypto.identity) {
                    match db.get_peer_key(&target) {
                        Ok(Some(peer_pk_bytes)) => {
                            let peer_pk = x25519_dalek::PublicKey::from(<[u8; 32]>::try_from(peer_pk_bytes).unwrap());
                            let shared_secret = CryptoManager::derive_shared_secret(&identity.x25519_secret, &peer_pk);
                            
                            match CryptoManager::encrypt_direct(plaintext.as_bytes(), &shared_secret) {
                                Ok(ciphertext) => {
                                    if let Ok(target_id) = target.parse::<iroh::EndpointId>() {
                                        let addr = iroh::EndpointAddr::from(target_id);
                                        if let Err(e) = net.send_direct(addr, ciphertext).await {
                                            let _ = self.tx_events.send(AppEvent::ErrorMessage(format!("Send failed: {}", e)));
                                        } else {
                                            println!("[Worker] E2EE message sent to {}", target);
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = self.tx_events.send(AppEvent::ErrorMessage(format!("Encryption failed: {}", e)));
                                }
                            }
                        }
                        _ => {
                            let _ = self.tx_events.send(AppEvent::ErrorMessage("Contact not found. Add them by ticket first.".into()));
                        }
                    }
                }
            }
            Command::CreateGroup { name } => {
                println!("[Worker] Creating local group swarm: {}", name);
            }
            _ => {
                // Forward-compatible catch for remaining variants
            }
        }
        Ok(())
    }
}
