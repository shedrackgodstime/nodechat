use crate::core::commands::{AppEvent, Command};
use eframe::egui;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Clone, PartialEq)]
pub enum ViewState {
    Welcome,
    SetupName,
    Generating,
    IdentityCard,
    MainChat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainTab {
    Chats,
    Contacts,
    Settings,
}

/// The main NodeChat UI Application state.
pub struct NodeChatApp {
    pub view_state: ViewState,
    pub current_tab: MainTab,
    pub display_name: String,
    pub local_node_id: Option<String>,
    pub local_ticket: Option<String>,

    // Chat state
    pub ticket_input: String,
    pub chat_input: String,
    pub contacts: Vec<String>,
    pub active_chat: Option<String>,
    pub messages: std::collections::HashMap<String, Vec<(String, String)>>, // sender, text

    // UI feedback
    pub copied_timer: f32,
    pub error_message: Option<(String, f32)>, // message, timer

    // Communication channels
    pub tx_cmd: mpsc::Sender<Command>,
    pub rx_event: broadcast::Receiver<AppEvent>,
}

impl NodeChatApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        tx_cmd: mpsc::Sender<Command>,
        rx_event: broadcast::Receiver<AppEvent>,
        has_identity: bool,
    ) -> Self {
        // Apply custom visual styling here (tokens from UX_FLOW.md)
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(28, 28, 30); // surface-primary
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(44, 44, 46); // surface-secondary
        _cc.egui_ctx.set_visuals(visuals);

        if has_identity {
            let _ = tx_cmd.try_send(Command::RefreshIdentity);
        }

        #[allow(unused_mut)]
        let mut view_state = if has_identity {
            ViewState::MainChat
        } else {
            ViewState::Welcome
        };

        #[cfg(target_os = "android")]
        {
            if !has_identity {
                let _ = tx_cmd.try_send(Command::UpdateProfile {
                    display_name: "Mobile".to_string(),
                });
                let _ = tx_cmd.try_send(Command::RefreshIdentity);
                view_state = ViewState::MainChat;
            }
        }

        Self {
            view_state,
            current_tab: MainTab::Chats,
            display_name: String::new(),
            local_node_id: None,
            local_ticket: None,
            ticket_input: String::new(),
            chat_input: String::new(),
            contacts: Vec::new(),
            active_chat: None,
            messages: std::collections::HashMap::new(),
            copied_timer: 0.0,
            error_message: None,
            tx_cmd,
            rx_event,
        }
    }
}

impl eframe::App for NodeChatApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Update feedback timers
        let dt = ui.ctx().input(|i| i.stable_dt);
        if self.copied_timer > 0.0 {
            self.copied_timer -= dt;
        }

        if let Some((_msg, timer)) = &mut self.error_message {
            *timer -= dt;
            if *timer <= 0.0 {
                self.error_message = None;
            }
        }

        // 1. Process backend events
        while let Ok(event) = self.rx_event.try_recv() {
            self.handle_event(event);
        }

        // 2. Render current view using the provided UI
        match self.view_state {
            ViewState::Welcome => self.render_welcome(ui),
            ViewState::SetupName => self.render_setup_name(ui),
            ViewState::Generating => self.render_generating(ui),
            ViewState::IdentityCard => self.render_identity_card(ui),
            ViewState::MainChat => self.render_main_chat(ui),
        }

        // 3. Render Error Toast Overlay
        self.render_error_toast(ui);

        // Ensure UI stays responsive by requesting repaint on events
        ui.ctx().request_repaint();
    }
}

impl NodeChatApp {
    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::IdentityCreated {
                node_id,
                ticket,
                display_name,
            } => {
                self.local_node_id = Some(node_id);
                self.local_ticket = Some(ticket);
                if let Some(name) = display_name {
                    self.display_name = name;
                }
                if self.view_state == ViewState::Generating {
                    self.view_state = ViewState::IdentityCard;
                }
            }
            AppEvent::IncomingMessage { sender, plaintext } => {
                let entry = self.messages.entry(sender.clone()).or_insert_with(Vec::new);
                entry.push((sender.clone(), plaintext));

                if !self.contacts.contains(&sender) {
                    self.contacts.push(sender);
                }
            }
            AppEvent::ErrorMessage(msg) => {
                self.error_message = Some((msg, 5.0));
            }
            AppEvent::IdentityGenerationFailed(msg) => {
                self.error_message = Some((format!("Identity gen failed: {}", msg), 5.0));
                self.view_state = ViewState::SetupName;
            }
            _ => {}
        }
    }

    // --- Screen Renderers ---

    fn render_welcome(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("NodeChat");
                ui.label("Secure. Private. Yours.");
                ui.add_space(40.0);

                if ui.button("Get Started").clicked() {
                    self.view_state = ViewState::SetupName;
                }

                ui.add_space(20.0);
                ui.label("No account. No phone number. No server.");
            });
        });
    }

    fn render_setup_name(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("Set up your identity");
                ui.add_space(20.0);
                ui.label("What should people call you?");

                ui.text_edit_singleline(&mut self.display_name);

                let is_valid = !self.display_name.trim().is_empty();
                ui.add_space(20.0);

                if ui
                    .add_enabled(is_valid, egui::Button::new("Continue"))
                    .clicked()
                {
                    let _ = self.tx_cmd.try_send(Command::UpdateProfile {
                        display_name: self.display_name.clone(),
                    });
                    self.view_state = ViewState::Generating;
                }
            });
        });
    }

    fn render_generating(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.spinner();
                ui.add_space(20.0);
                ui.heading("Generating your identity");
                ui.label("This happens once, on this device, entirely offline.");
            });
        });
    }

    fn render_identity_card(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading(format!("Hi, {}!", self.display_name));
                ui.label("Your identity is ready.");

                ui.add_space(20.0);
                ui.group(|ui| {
                    if let Some(node_id) = &self.local_node_id {
                        ui.label("YOUR NODE ID");
                        ui.code(node_id);
                        if ui.button("Copy ID").clicked() {
                            ui.ctx().copy_text(node_id.clone());
                            self.copied_timer = 2.0;
                        }
                    }

                    ui.add_space(10.0);

                    if let Some(ticket) = &self.local_ticket {
                        ui.label("INVITATION TICKET (Share this)");
                        ui.code(ticket);
                        if ui.button("Copy Ticket").clicked() {
                            ui.ctx().copy_text(ticket.clone());
                            self.copied_timer = 2.0;
                        }
                    }
                });

                if self.copied_timer > 0.0 {
                    ui.label(
                        egui::RichText::new("✓ Copied to clipboard").color(egui::Color32::GREEN),
                    );
                }

                ui.add_space(30.0);
                if ui.button("Go to Chats").clicked() {
                    self.view_state = ViewState::MainChat;
                }
            });
        });
    }

    fn render_main_chat(&mut self, ui: &mut egui::Ui) {
        let is_mobile = ui.available_width() < 600.0;

        if is_mobile {
            self.render_mobile_main(ui);
        } else {
            self.render_desktop_main(ui);
        }
    }

    fn render_mobile_main(&mut self, ui: &mut egui::Ui) {
        if let Some(peer_id) = self.active_chat.clone() {
            // Mobile Chat View
            egui::Panel::top("mobile_chat_header").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("←").clicked() {
                        self.active_chat = None;
                    }
                    ui.heading(format!("Chat with {}", peer_id));
                });
            });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.render_chat_window(ui, peer_id);
            });
        } else {
            // Mobile Home Screen
            self.render_bottom_bar(ui);

            egui::CentralPanel::default().show_inside(ui, |ui| match self.current_tab {
                MainTab::Chats => self.render_chats_list(ui),
                MainTab::Contacts => self.render_contacts_page(ui),
                MainTab::Settings => self.render_settings_page(ui),
            });
        }
    }

    fn render_desktop_main(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("contacts_sidebar").show_inside(ui, |ui| {
            ui.heading("NodeChat");
            ui.separator();

            if ui
                .selectable_label(self.current_tab == MainTab::Chats, "Chats")
                .clicked()
            {
                self.current_tab = MainTab::Chats;
            }
            if ui
                .selectable_label(self.current_tab == MainTab::Contacts, "Contacts")
                .clicked()
            {
                self.current_tab = MainTab::Contacts;
            }
            if ui
                .selectable_label(self.current_tab == MainTab::Settings, "Settings")
                .clicked()
            {
                self.current_tab = MainTab::Settings;
            }

            ui.separator();
            match self.current_tab {
                MainTab::Chats => self.render_chats_list(ui),
                MainTab::Contacts => self.render_contacts_page(ui),
                MainTab::Settings => self.render_settings_page(ui),
            }
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(peer_id) = &self.active_chat {
                self.render_chat_window(ui, peer_id.clone());
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Welcome to NodeChat");
                    ui.label("Pick a conversation to start messaging.");

                    if let Some(ticket) = &self.local_ticket {
                        if ui.button("Copy My Ticket").clicked() {
                            ui.ctx().copy_text(ticket.clone());
                            self.copied_timer = 2.0;
                        }
                    }
                });
            }
        });
    }

    fn render_bottom_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("mobile_tabs").show_inside(ui, |ui| {
            ui.horizontal_centered(|ui| {
                let tab_width = (ui.available_width() - 40.0) / 3.0;

                if ui
                    .add_sized(
                        [tab_width, 40.0],
                        egui::Button::selectable(self.current_tab == MainTab::Chats, "Chats"),
                    )
                    .clicked()
                {
                    self.current_tab = MainTab::Chats;
                }

                if ui
                    .add_sized(
                        [tab_width, 40.0],
                        egui::Button::selectable(self.current_tab == MainTab::Contacts, "Contacts"),
                    )
                    .clicked()
                {
                    self.current_tab = MainTab::Contacts;
                }

                if ui
                    .add_sized(
                        [tab_width, 40.0],
                        egui::Button::selectable(self.current_tab == MainTab::Settings, "Settings"),
                    )
                    .clicked()
                {
                    self.current_tab = MainTab::Settings;
                }
            });
        });
    }

    fn render_chats_list(&mut self, ui: &mut egui::Ui) {
        ui.heading("Chats");
        ui.separator();

        if self.contacts.is_empty() {
            ui.label("No active chats yet.");
        } else {
            for contact in &self.contacts {
                let is_selected = Some(contact.clone()) == self.active_chat;
                if ui.selectable_label(is_selected, contact).clicked() {
                    self.active_chat = Some(contact.clone());
                }
            }
        }
    }

    fn render_contacts_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Manage Contacts");
        ui.add_space(10.0);
        ui.label("Add Peer by Ticket:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.ticket_input);
            if ui.button("Connect").clicked() && !self.ticket_input.is_empty() {
                let _ = self.tx_cmd.try_send(Command::AddContactByTicket {
                    ticket: self.ticket_input.clone(),
                });
                self.ticket_input.clear();
            }
        });

        ui.separator();
        ui.label("Saved Contacts will appear here.");
    }

    fn render_settings_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();

        ui.label("YOUR PROFILE");
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.label(&self.display_name);
        });

        ui.add_space(10.0);
        ui.label("IDENTITY");
        if let Some(node_id) = &self.local_node_id {
            ui.label("Node ID:");
            ui.code(node_id);
            if ui.button("Copy Node ID").clicked() {
                ui.ctx().copy_text(node_id.clone());
                self.copied_timer = 2.0;
            }
        }

        ui.add_space(10.0);
        if let Some(ticket) = &self.local_ticket {
            ui.label("My Chat Ticket:");
            ui.code(ticket);
            if ui.button("Copy My Ticket").clicked() {
                ui.ctx().copy_text(ticket.clone());
                self.copied_timer = 2.0;
            }
        }

        if self.copied_timer > 0.0 {
            ui.add_space(10.0);
            ui.label(egui::RichText::new("✓ Copied").color(egui::Color32::GREEN));
        }
    }

    fn render_chat_window(&mut self, ui: &mut egui::Ui, peer_id: String) {
        ui.vertical(|ui| {
            // Scrollable message area
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if let Some(history) = self.messages.get(&peer_id) {
                        for (sender, text) in history {
                            let is_me = sender == "Me";
                            ui.horizontal(|ui| {
                                if is_me {
                                    ui.add_space(ui.available_width() * 0.2);
                                }

                                egui::Frame::group(ui.style())
                                    .fill(if is_me {
                                        egui::Color32::from_rgb(26, 95, 168)
                                    } else {
                                        egui::Color32::from_rgb(44, 44, 46)
                                    })
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(text).color(egui::Color32::WHITE),
                                        );
                                    });

                                if !is_me {
                                    ui.add_space(ui.available_width() * 0.2);
                                }
                            });
                            ui.add_space(5.0);
                        }
                    }
                });

            ui.separator();
            // Input area
            let response = ui.text_edit_singleline(&mut self.chat_input);
            if response.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !self.chat_input.is_empty()
            {
                let _ = self.tx_cmd.try_send(Command::SendDirectMessage {
                    target: peer_id.clone(),
                    plaintext: self.chat_input.clone(),
                });

                // Optimistically update UI
                let entry = self
                    .messages
                    .entry(peer_id.clone())
                    .or_insert_with(Vec::new);
                entry.push(("Me".to_string(), self.chat_input.clone()));
                self.chat_input.clear();

                // Focus the input again
                ui.memory_mut(|mem| mem.request_focus(response.id));
            }
        });
    }

    fn render_error_toast(&mut self, ui: &mut egui::Ui) {
        if let Some((msg_content, _)) = &self.error_message {
            let msg = msg_content.clone();

            egui::Area::new(egui::Id::new("error_toast"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
                .show(ui.ctx(), |ui| {
                    egui::Frame::window(&ui.style())
                        .fill(egui::Color32::from_rgb(120, 20, 20))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::RED))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("⚠")
                                        .strong()
                                        .color(egui::Color32::WHITE),
                                );
                                ui.label(egui::RichText::new(msg).color(egui::Color32::WHITE));
                                if ui.button("X").clicked() {
                                    self.error_message = None;
                                }
                            });
                        });
                });
        }
    }
}
