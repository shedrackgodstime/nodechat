use eframe::egui;
use tokio::sync::{mpsc, broadcast};
use crate::core::commands::{Command, AppEvent};

pub struct NodeChatUI {
    tx_cmd: mpsc::Sender<Command>,
    rx_event: broadcast::Receiver<AppEvent>,
    status: String,
    
    // Simple state for Phase 1
    target_node: String,
    message_draft: String,
}

impl NodeChatUI {
    pub fn new(
        tx_cmd: mpsc::Sender<Command>, 
        rx_event: broadcast::Receiver<AppEvent>
    ) -> Self {
        Self {
            tx_cmd,
            rx_event,
            status: "Initializing backend...".to_string(),
            target_node: String::new(),
            message_draft: String::new(),
        }
    }
    
    // Handle background events non-blockingly
    fn process_events(&mut self) {
        while let Ok(event) = self.rx_event.try_recv() {
            match event {
                AppEvent::BackendReady => {
                    self.status = "Backend Online (P2P Ready)".to_string();
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for NodeChatUI {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.process_events();

        // Left Panel - Contacts & Groups 
        egui::Panel::left("contacts_panel").show_inside(ui, |ui| {
            ui.heading("Phonebook & Swarms");
            ui.label(format!("State: {}", self.status));
            ui.separator();
            ui.label("Decentralized address book will load here.");
        });

        // Center Panel - Chat execution
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("NodeChat Secure Comm");
            ui.label("E2EE Direct & Group messaging framework.");
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Target NodeId:");
                ui.text_edit_singleline(&mut self.target_node);
            });
            
            ui.horizontal(|ui| {
                ui.label("Message:");
                ui.text_edit_singleline(&mut self.message_draft);
            });
            
            if ui.button("Send Direct Message").clicked() {
                let cmd = Command::SendDirectMessage { 
                    target: self.target_node.clone(), 
                    plaintext: self.message_draft.clone() 
                };
                let _ = self.tx_cmd.try_send(cmd);
                self.message_draft.clear();
            }
        });
    }

    /// Clean shutdown hook
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.tx_cmd.try_send(Command::Quit);
    }
}
