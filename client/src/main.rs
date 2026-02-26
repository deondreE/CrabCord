use eframe::egui;
use egui::{Color32, FontId, RichText, Rounding, Vec2};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Message {
    id: String,
    username: Option<String>,
    content: String,
}

#[derive(Serialize)]
struct MessageRequest {
    content: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct User {
    id: String,
    username: String,
}

#[derive(Deserialize, Clone, Debug)]
struct LoginResponse {
    token: String,
    #[allow(dead_code)]
    refresh_token: Option<String>,
    user: User,
}

#[derive(Deserialize, Clone, Debug)]
struct Server {
    id: String,
    name: String,
}

#[derive(Deserialize, Clone, Debug)]
struct Channel {
    id: String,
    name: String,
}

#[derive(Deserialize, Clone, Debug)]
struct VoiceParticipant {
    #[allow(dead_code)]
    user_id: String,
    username: String,
    muted: bool,
    deafened: bool,
}

#[derive(Deserialize, Clone, Debug)]
struct VoiceChannel {
    id: String,
    name: String,
    max_users: Option<usize>,
    participants: Vec<VoiceParticipant>,
}

#[derive(Debug)]
struct LoginResult {
    token: String,
    user: User,
}

enum AsyncMsg {
    LoginOk(LoginResult),
    LoginErr(String),
    RegisterOk(LoginResult),
    RegisterErr(String),
    ServersOk(Vec<Server>),
    ChannelsOk(Vec<Channel>),
    VoiceChannelsOk(Vec<VoiceChannel>),
    MessagesOk(Vec<Message>),
    VoiceJoinOk(String),
    VoiceLeaveOk,
    VoiceStateOk,
    ApiErr(String),
}

#[derive(Clone, Default)]
struct VoiceSession {
    channel_id: String,
    channel_name: String,
    muted: bool,
    deafened: bool,
}

#[derive(PartialEq, Clone)]
enum AppView {
    Login,
    Register,
    Chat,
}

struct CrabCordApp {
    view: AppView,
    email_input: String,
    password_input: String,
    username_input: String,
    confirm_password_input: String,
    auth_error: String,
    auth_loading: bool,
    token: String,
    current_user: Option<User>,
    servers: Vec<Server>,
    selected_server_id: String,
    create_server_input: String,
    show_create_server: bool,
    channels: Vec<Channel>,
    selected_channel_id: String,
    selected_channel_name: String,
    voice_channels: Vec<VoiceChannel>,
    voice_session: Option<VoiceSession>,
    messages: Vec<Message>,
    message_input: String,
    last_poll: Instant,
    last_voice_poll: Instant,
    tx: Sender<AsyncMsg>,
    rx: Receiver<AsyncMsg>,
}

impl Default for CrabCordApp {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            view: AppView::Login,
            email_input: String::new(),
            password_input: String::new(),
            username_input: String::new(),
            confirm_password_input: String::new(),
            auth_error: String::new(),
            auth_loading: false,
            token: String::new(),
            current_user: None,
            servers: Vec::new(),
            selected_server_id: String::new(),
            create_server_input: String::new(),
            show_create_server: false,
            channels: Vec::new(),
            selected_channel_id: String::new(),
            selected_channel_name: String::new(),
            voice_channels: Vec::new(),
            voice_session: None,
            messages: Vec::new(),
            message_input: String::new(),
            last_poll: Instant::now() - Duration::from_secs(10),
            last_voice_poll: Instant::now() - Duration::from_secs(10),
            tx,
            rx,
        }
    }
}

const BG_DARKEST: Color32 = Color32::from_rgb(15, 15, 20);
const BG_SERVERS: Color32 = Color32::from_rgb(20, 20, 28);
const BG_SIDEBAR: Color32 = Color32::from_rgb(28, 28, 38);
const BG_CHAT: Color32 = Color32::from_rgb(34, 34, 46);
const BG_INPUT: Color32 = Color32::from_rgb(42, 42, 58);
const ACCENT: Color32 = Color32::from_rgb(88, 101, 242);
const ACCENT_HOVER: Color32 = Color32::from_rgb(108, 121, 255);
const ACCENT_DIM: Color32 = Color32::from_rgb(55, 63, 150);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(220, 221, 228);
const TEXT_DIM: Color32 = Color32::from_rgb(150, 154, 178);
const TEXT_MUTED: Color32 = Color32::from_rgb(80, 83, 105);
const GREEN: Color32 = Color32::from_rgb(87, 242, 135);
const RED: Color32 = Color32::from_rgb(240, 71, 71);
const YELLOW: Color32 = Color32::from_rgb(250, 168, 26);
const VOICE_BG: Color32 = Color32::from_rgb(22, 38, 28);
const VOICE_ACCENT: Color32 = Color32::from_rgb(87, 242, 135);

const BASE: &str = "http://localhost:3000";

fn auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}

impl CrabCordApp {
    fn perform_login(&mut self) {
        self.auth_loading = true;
        self.auth_error.clear();
        let tx = self.tx.clone();
        let email = self.email_input.clone();
        let password = self.password_input.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            match client
                .post(format!("{}/auth/login", BASE))
                .json(&serde_json::json!({ "email": email, "password": password }))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<LoginResponse>().await {
                        Ok(data) => {
                            let _ = tx.send(AsyncMsg::LoginOk(LoginResult {
                                token: data.token,
                                user: data.user,
                            }));
                        }
                        Err(e) => {
                            let _ = tx.send(AsyncMsg::LoginErr(e.to_string()));
                        }
                    }
                }
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    let msg = if status.as_u16() == 401 {
                        "Invalid email or password.".to_string()
                    } else {
                        format!("Login failed ({}): {}", status, body)
                    };
                    let _ = tx.send(AsyncMsg::LoginErr(msg));
                }
                Err(e) => {
                    let _ = tx.send(AsyncMsg::LoginErr(e.to_string()));
                }
            }
        });
    }

    fn perform_register(&mut self) {
        if self.password_input != self.confirm_password_input {
            self.auth_error = "Passwords do not match.".to_string();
            return;
        }
        self.auth_loading = true;
        self.auth_error.clear();
        let tx = self.tx.clone();
        let email = self.email_input.clone();
        let password = self.password_input.clone();
        let username = self.username_input.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            match client
                .post(format!("{}/auth/register", BASE))
                .json(&serde_json::json!({ "username": username, "email": email, "password": password }))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<LoginResponse>().await {
                        Ok(data) => { let _ = tx.send(AsyncMsg::RegisterOk(LoginResult { token: data.token, user: data.user })); }
                        Err(e)   => { let _ = tx.send(AsyncMsg::RegisterErr(e.to_string())); }
                    }
                }
                Ok(resp) => {
                    let body = resp.text().await.unwrap_or_default();
                    let _ = tx.send(AsyncMsg::RegisterErr(format!("Registration failed: {}", body)));
                }
                Err(e) => { let _ = tx.send(AsyncMsg::RegisterErr(e.to_string())); }
            }
        });
    }

    fn after_auth(&mut self, result: LoginResult) {
        self.token = result.token;
        self.current_user = Some(result.user);
        self.auth_loading = false;
        self.view = AppView::Chat;
        self.fetch_servers();
    }

    fn fetch_servers(&self) {
        let tx = self.tx.clone();
        let token = self.token.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(resp) = client
                .get(format!("{}/servers", BASE))
                .header("Authorization", auth_header(&token))
                .send()
                .await
            {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<Vec<Server>>().await {
                        let _ = tx.send(AsyncMsg::ServersOk(data));
                    }
                }
            }
        });
    }

    fn fetch_channels(&self, server_id: &str) {
        let tx = self.tx.clone();
        let token = self.token.clone();
        let server_id = server_id.to_string();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(resp) = client
                .get(format!("{}/servers/{}/channels", BASE, server_id))
                .header("Authorization", auth_header(&token))
                .send()
                .await
            {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<Vec<Channel>>().await {
                        let _ = tx.send(AsyncMsg::ChannelsOk(data));
                    }
                }
            }
        });
    }

    fn fetch_voice_channels(&self, server_id: &str) {
        let tx = self.tx.clone();
        let token = self.token.clone();
        let server_id = server_id.to_string();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(resp) = client
                .get(format!("{}/servers/{}/voice", BASE, server_id))
                .header("Authorization", auth_header(&token))
                .send()
                .await
            {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<Vec<VoiceChannel>>().await {
                        let _ = tx.send(AsyncMsg::VoiceChannelsOk(data));
                    }
                }
            }
        });
    }

    fn fetch_messages(&self) {
        if self.selected_channel_id.is_empty() {
            return;
        }
        let tx = self.tx.clone();
        let token = self.token.clone();
        let channel_id = self.selected_channel_id.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(resp) = client
                .get(format!("{}/channels/{}/messages", BASE, channel_id))
                .header("Authorization", auth_header(&token))
                .send()
                .await
            {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<Vec<Message>>().await {
                        let _ = tx.send(AsyncMsg::MessagesOk(data));
                    }
                }
            }
        });
    }

    fn send_message(&mut self) {
        let content = self.message_input.trim().to_string();
        if content.is_empty() || self.selected_channel_id.is_empty() {
            return;
        }
        self.message_input.clear();
        let tx = self.tx.clone();
        let token = self.token.clone();
        let channel_id = self.selected_channel_id.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(resp) = client
                .post(format!("{}/channels/{}/messages", BASE, channel_id))
                .header("Authorization", auth_header(&token))
                .json(&MessageRequest { content })
                .send()
                .await
            {
                if resp.status().is_success() {
                    if let Ok(fetch) = client
                        .get(format!("{}/channels/{}/messages", BASE, channel_id))
                        .header("Authorization", auth_header(&token))
                        .send()
                        .await
                    {
                        if let Ok(data) = fetch.json::<Vec<Message>>().await {
                            let _ = tx.send(AsyncMsg::MessagesOk(data));
                        }
                    }
                }
            }
        });
    }

    fn create_server(&mut self) {
        let name = self.create_server_input.trim().to_string();
        if name.is_empty() {
            return;
        }
        self.create_server_input.clear();
        self.show_create_server = false;
        let tx = self.tx.clone();
        let token = self.token.clone();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(resp) = client
                .post(format!("{}/servers", BASE))
                .header("Authorization", auth_header(&token))
                .json(&serde_json::json!({ "name": name }))
                .send()
                .await
            {
                if resp.status().is_success() {
                    if let Ok(list) = client
                        .get(format!("{}/servers", BASE))
                        .header("Authorization", auth_header(&token))
                        .send()
                        .await
                    {
                        if let Ok(data) = list.json::<Vec<Server>>().await {
                            let _ = tx.send(AsyncMsg::ServersOk(data));
                        }
                    }
                }
            }
        });
    }

    fn join_voice(&mut self, vc_id: &str, vc_name: &str) {
        let tx = self.tx.clone();
        let token = self.token.clone();
        let vc_id = vc_id.to_string();
        let vc_name = vc_name.to_string();
        tokio::spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(resp) = client
                .post(format!("{}/voice/{}/join", BASE, vc_id))
                .header("Authorization", auth_header(&token))
                .send()
                .await
            {
                if resp.status().is_success() {
                    let _ = tx.send(AsyncMsg::VoiceJoinOk(format!("{}|{}", vc_id, vc_name)));
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    let _ = tx.send(AsyncMsg::ApiErr(format!("Join voice failed: {}", body)));
                }
            }
        });
    }

    fn leave_voice(&mut self) {
        if let Some(session) = self.voice_session.take() {
            let tx = self.tx.clone();
            let token = self.token.clone();
            tokio::spawn(async move {
                let client = reqwest::Client::new();
                let _ = client
                    .post(format!("{}/voice/{}/leave", BASE, session.channel_id))
                    .header("Authorization", auth_header(&token))
                    .send()
                    .await;
                let _ = tx.send(AsyncMsg::VoiceLeaveOk);
            });
        }
    }

    fn toggle_mute(&mut self) {
        if let Some(s) = self.voice_session.as_mut() {
            s.muted = !s.muted;
            let (tx, token, vc_id, muted) = (
                self.tx.clone(),
                self.token.clone(),
                s.channel_id.clone(),
                s.muted,
            );
            tokio::spawn(async move {
                let client = reqwest::Client::new();
                let _ = client
                    .patch(format!("{}/voice/{}/state", BASE, vc_id))
                    .header("Authorization", auth_header(&token))
                    .json(&serde_json::json!({ "muted": muted }))
                    .send()
                    .await;
                let _ = tx.send(AsyncMsg::VoiceStateOk);
            });
        }
    }

    fn toggle_deafen(&mut self) {
        if let Some(s) = self.voice_session.as_mut() {
            s.deafened = !s.deafened;
            let (tx, token, vc_id, deafened) = (
                self.tx.clone(),
                self.token.clone(),
                s.channel_id.clone(),
                s.deafened,
            );
            tokio::spawn(async move {
                let client = reqwest::Client::new();
                let _ = client
                    .patch(format!("{}/voice/{}/state", BASE, vc_id))
                    .header("Authorization", auth_header(&token))
                    .json(&serde_json::json!({ "deafened": deafened }))
                    .send()
                    .await;
                let _ = tx.send(AsyncMsg::VoiceStateOk);
            });
        }
    }

    // ── UI ─────────────────────────────────────────────────────────────────

    fn show_login(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_DARKEST))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(90.0);
                    ui.label(RichText::new("🦀").size(52.0));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("CrabCord")
                            .size(28.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    ui.label(RichText::new("Welcome back!").size(14.0).color(TEXT_DIM));
                    ui.add_space(24.0);

                    egui::Frame::none()
                        .fill(BG_SIDEBAR)
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(28.0))
                        .show(ui, |ui| {
                            ui.set_min_width(340.0);
                            ui.set_max_width(340.0);

                            field_label(ui, "EMAIL");
                            ui.add(text_field(&mut self.email_input, false));
                            ui.add_space(14.0);

                            field_label(ui, "PASSWORD");
                            let pw = ui.add(text_field(&mut self.password_input, true));
                            ui.add_space(20.0);

                            show_error(ui, &self.auth_error);

                            let lbl = if self.auth_loading {
                                "Logging in…"
                            } else {
                                "Log In"
                            };
                            let clicked = accent_btn_full(ui, lbl, 40.0).clicked();
                            let enter =
                                pw.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if (clicked || enter) && !self.auth_loading {
                                self.perform_login();
                            }

                            ui.add_space(16.0);
                            ui.separator();
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Need an account?").color(TEXT_DIM).size(13.0),
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("Register").color(ACCENT).size(13.0),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    self.view = AppView::Register;
                                    self.auth_error.clear();
                                }
                            });
                        });
                });
            });
    }

    fn show_register(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_DARKEST))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.label(RichText::new("🦀").size(52.0));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("CrabCord")
                            .size(28.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new("Create an account")
                            .size(14.0)
                            .color(TEXT_DIM),
                    );
                    ui.add_space(24.0);

                    egui::Frame::none()
                        .fill(BG_SIDEBAR)
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(28.0))
                        .show(ui, |ui| {
                            ui.set_min_width(340.0);
                            ui.set_max_width(340.0);

                            field_label(ui, "USERNAME");
                            ui.add(text_field(&mut self.username_input, false));
                            ui.add_space(14.0);

                            field_label(ui, "EMAIL");
                            ui.add(text_field(&mut self.email_input, false));
                            ui.add_space(14.0);

                            field_label(ui, "PASSWORD");
                            ui.add(text_field(&mut self.password_input, true));
                            ui.add_space(14.0);

                            field_label(ui, "CONFIRM PASSWORD");
                            let confirm =
                                ui.add(text_field(&mut self.confirm_password_input, true));
                            ui.add_space(20.0);

                            show_error(ui, &self.auth_error);

                            let lbl = if self.auth_loading {
                                "Creating account…"
                            } else {
                                "Create Account"
                            };
                            let clicked = accent_btn_full(ui, lbl, 40.0).clicked();
                            let enter = confirm.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if (clicked || enter) && !self.auth_loading {
                                self.perform_register();
                            }

                            ui.add_space(16.0);
                            ui.separator();
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Already have an account?")
                                        .color(TEXT_DIM)
                                        .size(13.0),
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("Log In").color(ACCENT).size(13.0),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    self.view = AppView::Login;
                                    self.auth_error.clear();
                                }
                            });
                        });
                });
            });
    }

    fn show_chat(&mut self, ctx: &egui::Context) {
        if self.voice_session.is_some() {
            egui::TopBottomPanel::bottom("voice_bar")
                .frame(
                    egui::Frame::none()
                        .fill(VOICE_BG)
                        .inner_margin(egui::Margin::same(10.0)),
                )
                .min_height(56.0)
                .show(ctx, |ui| {
                    self.show_voice_bar(ui);
                });
        }

        egui::TopBottomPanel::bottom("input_bar")
            .frame(
                egui::Frame::none()
                    .fill(BG_CHAT)
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0)),
            )
            .min_height(56.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let hint = if self.selected_channel_id.is_empty() {
                        "Select a channel to chat".to_string()
                    } else {
                        format!("Message #{}", self.selected_channel_name)
                    };
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.message_input)
                            .hint_text(hint)
                            .desired_width(ui.available_width() - 72.0)
                            .font(FontId::proportional(14.0)),
                    );
                    let send = accent_btn_sized(ui, "Send", Vec2::new(64.0, 36.0));
                    if (send.clicked()
                        || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                        && !self.selected_channel_id.is_empty()
                    {
                        self.send_message();
                    }
                });
            });

        egui::SidePanel::left("server_panel")
            .frame(
                egui::Frame::none()
                    .fill(BG_SERVERS)
                    .inner_margin(egui::Margin::symmetric(8.0, 12.0)),
            )
            .exact_width(68.0)
            .show(ctx, |ui| {
                self.show_server_list(ui);
            });

        egui::SidePanel::left("channel_panel")
            .frame(egui::Frame::none().fill(BG_SIDEBAR))
            .min_width(200.0)
            .max_width(240.0)
            .show(ctx, |ui| {
                self.show_channel_sidebar(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_CHAT))
            .show(ctx, |ui| {
                self.show_messages(ui);
            });
    }

    fn show_voice_bar(&mut self, ui: &mut egui::Ui) {
        let (name, muted, deafened) = self
            .voice_session
            .as_ref()
            .map(|s| (s.channel_name.clone(), s.muted, s.deafened))
            .unwrap_or_default();

        ui.horizontal(|ui| {
            let dot = ui
                .allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover())
                .0;
            ui.painter().circle_filled(dot.center(), 4.0, VOICE_ACCENT);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Voice Connected")
                        .size(11.0)
                        .color(VOICE_ACCENT),
                );
                ui.label(
                    RichText::new(format!("🔊 {}", name))
                        .size(13.0)
                        .color(TEXT_PRIMARY),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(icon_btn("✕", RED, "Disconnect")).clicked() {
                    self.leave_voice();
                }
                ui.add_space(4.0);
                if ui
                    .add(icon_btn(
                        "🔇",
                        if deafened { YELLOW } else { TEXT_DIM },
                        if deafened { "Undeafen" } else { "Deafen" },
                    ))
                    .clicked()
                {
                    self.toggle_deafen();
                }
                ui.add_space(4.0);
                if ui
                    .add(icon_btn(
                        if muted { "🔴" } else { "🎤" },
                        if muted { RED } else { GREEN },
                        if muted { "Unmute" } else { "Mute" },
                    ))
                    .clicked()
                {
                    self.toggle_mute();
                }
            });
        });
    }

    fn show_server_list(&mut self, ui: &mut egui::Ui) {
        let servers = self.servers.clone();
        let selected = self.selected_server_id.clone();

        for server in &servers {
            let is_sel = server.id == selected;
            let initial = server
                .name
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .next()
                .unwrap_or('?');
            let (rect, resp) = ui.allocate_exact_size(Vec2::splat(48.0), egui::Sense::click());

            if is_sel {
                let pill = egui::Rect::from_min_size(
                    rect.left_top() + egui::vec2(-8.0, 12.0),
                    Vec2::new(4.0, 24.0),
                );
                ui.painter().rect_filled(pill, Rounding::same(2.0), ACCENT);
            }
            let bg = if is_sel {
                ACCENT
            } else if resp.hovered() {
                ACCENT_DIM
            } else {
                Color32::from_rgb(40, 40, 55)
            };
            let rnd = if is_sel {
                Rounding::same(16.0)
            } else {
                Rounding::same(24.0)
            };
            ui.painter().rect_filled(rect, rnd, bg);
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                initial.to_string(),
                FontId::proportional(18.0),
                TEXT_PRIMARY,
            );

            if resp.on_hover_text(&server.name).clicked() && !is_sel {
                let sid = server.id.clone();
                self.selected_server_id = sid.clone();
                self.selected_channel_id.clear();
                self.selected_channel_name.clear();
                self.messages.clear();
                self.fetch_channels(&sid);
                self.fetch_voice_channels(&sid);
            }
            ui.add_space(6.0);
        }

        ui.add_space(4.0);
        let sep = ui
            .allocate_exact_size(Vec2::new(32.0, 1.0), egui::Sense::hover())
            .0;
        ui.painter().rect_filled(sep, Rounding::ZERO, TEXT_MUTED);
        ui.add_space(4.0);

        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(48.0), egui::Sense::click());
        let (bg, fg) = if resp.hovered() {
            (GREEN, BG_DARKEST)
        } else {
            (Color32::from_rgb(40, 40, 55), GREEN)
        };
        ui.painter().rect_filled(rect, Rounding::same(24.0), bg);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "+",
            FontId::proportional(22.0),
            fg,
        );
        if resp.on_hover_text("Create Server").clicked() {
            self.show_create_server = true;
        }

        if self.show_create_server {
            egui::Window::new("Create Server")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(
                    egui::Frame::none()
                        .fill(BG_SIDEBAR)
                        .rounding(Rounding::same(12.0))
                        .inner_margin(egui::Margin::same(20.0)),
                )
                .show(ui.ctx(), |ui| {
                    field_label(ui, "SERVER NAME");
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.create_server_input)
                            .desired_width(260.0)
                            .font(FontId::proportional(14.0)),
                    );
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        let create = accent_btn_sized(ui, "Create", Vec2::new(120.0, 32.0));
                        let cancel = ui.add_sized(
                            [80.0, 32.0],
                            egui::Button::new(RichText::new("Cancel").color(TEXT_DIM).size(14.0))
                                .fill(Color32::from_rgb(40, 40, 55)),
                        );
                        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if create.clicked() || enter {
                            self.create_server();
                        }
                        if cancel.clicked() {
                            self.show_create_server = false;
                            self.create_server_input.clear();
                        }
                    });
                });
        }
    }

    fn show_channel_sidebar(&mut self, ui: &mut egui::Ui) {
        let server_name = self
            .servers
            .iter()
            .find(|s| s.id == self.selected_server_id)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "No server selected".to_string());

        egui::Frame::none()
            .fill(BG_SIDEBAR)
            .inner_margin(egui::Margin::symmetric(14.0, 14.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(&server_name)
                        .strong()
                        .size(15.0)
                        .color(TEXT_PRIMARY),
                );
            });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);

            if !self.channels.is_empty() {
                section_header(ui, "TEXT CHANNELS");
                let channels = self.channels.clone();
                let sel = self.selected_channel_id.clone();
                for ch in &channels {
                    let is_sel = ch.id == sel;
                    if channel_row(ui, &format!("# {}", ch.name), is_sel).clicked() && !is_sel {
                        self.selected_channel_id = ch.id.clone();
                        self.selected_channel_name = ch.name.clone();
                        self.messages.clear();
                        self.last_poll = Instant::now() - Duration::from_secs(10);
                    }
                }
                ui.add_space(8.0);
            }

            if !self.voice_channels.is_empty() {
                section_header(ui, "VOICE CHANNELS");
                let vcs = self.voice_channels.clone();
                let active = self
                    .voice_session
                    .as_ref()
                    .map(|s| s.channel_id.clone())
                    .unwrap_or_default();
                for vc in &vcs {
                    let is_active = vc.id == active;
                    let color = if is_active { VOICE_ACCENT } else { TEXT_DIM };
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(10.0, 2.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let resp = ui.add(
                                    egui::Label::new(
                                        RichText::new(format!("🔊 {}", vc.name))
                                            .size(13.5)
                                            .color(color),
                                    )
                                    .sense(egui::Sense::click()),
                                );
                                if let Some(max) = vc.max_users {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(format!(
                                                    "{}/{}",
                                                    vc.participants.len(),
                                                    max
                                                ))
                                                .size(11.0)
                                                .color(TEXT_MUTED),
                                            );
                                        },
                                    );
                                } else if !vc.participants.is_empty() {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(format!("{}", vc.participants.len()))
                                                    .size(11.0)
                                                    .color(TEXT_MUTED),
                                            );
                                        },
                                    );
                                }
                                if resp.clicked() {
                                    if is_active {
                                        self.leave_voice();
                                    } else {
                                        self.join_voice(&vc.id, &vc.name);
                                    }
                                }
                            });
                            for p in &vc.participants {
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
                                    let (cr, _) = ui.allocate_exact_size(
                                        Vec2::splat(16.0),
                                        egui::Sense::hover(),
                                    );
                                    let init = p
                                        .username
                                        .chars()
                                        .next()
                                        .unwrap_or('?')
                                        .to_uppercase()
                                        .next()
                                        .unwrap_or('?');
                                    ui.painter().circle_filled(cr.center(), 8.0, ACCENT_DIM);
                                    ui.painter().text(
                                        cr.center(),
                                        egui::Align2::CENTER_CENTER,
                                        init.to_string(),
                                        FontId::proportional(9.0),
                                        TEXT_PRIMARY,
                                    );
                                    let nc = if p.muted || p.deafened {
                                        TEXT_MUTED
                                    } else {
                                        TEXT_DIM
                                    };
                                    ui.label(RichText::new(&p.username).size(12.0).color(nc));
                                    if p.muted {
                                        ui.label(RichText::new("🔴").size(10.0));
                                    }
                                    if p.deafened {
                                        ui.label(RichText::new("🔇").size(10.0));
                                    }
                                });
                            }
                        });
                    ui.add_space(2.0);
                }
            }
            ui.add_space(8.0);
        });

        if let Some(user) = &self.current_user {
            egui::TopBottomPanel::bottom("user_info_bar")
                .frame(
                    egui::Frame::none()
                        .fill(Color32::from_rgb(20, 20, 30))
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0)),
                )
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::splat(32.0), egui::Sense::hover());
                        let init = user
                            .username
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .next()
                            .unwrap_or('?');
                        ui.painter().circle_filled(rect.center(), 16.0, ACCENT);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            init.to_string(),
                            FontId::proportional(14.0),
                            TEXT_PRIMARY,
                        );
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&user.username)
                                    .size(13.0)
                                    .strong()
                                    .color(TEXT_PRIMARY),
                            );
                            ui.label(RichText::new("Online").size(11.0).color(GREEN));
                        });
                    });
                });
        }
    }

    fn show_messages(&self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("chat_header")
            .frame(
                egui::Frame::none()
                    .fill(BG_CHAT)
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
            )
            .show_inside(ui, |ui| {
                if self.selected_channel_id.is_empty() {
                    ui.label(RichText::new("Select a channel").size(15.0).color(TEXT_DIM));
                } else {
                    ui.label(
                        RichText::new(format!("# {}", self.selected_channel_name))
                            .size(16.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                }
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(BG_CHAT)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
            )
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.messages.is_empty() {
                            ui.add_space(60.0);
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new(if self.selected_channel_id.is_empty() {
                                        "👈 Select a channel to start chatting"
                                    } else {
                                        "No messages yet. Be the first to say something!"
                                    })
                                    .color(TEXT_DIM)
                                    .size(14.0),
                                );
                            });
                        } else {
                            for (i, msg) in self.messages.iter().enumerate() {
                                let prev = if i > 0 {
                                    self.messages[i - 1].username.as_deref()
                                } else {
                                    None
                                };
                                let author = msg.username.as_deref().unwrap_or("Unknown");
                                let grouped = prev == Some(author);

                                if !grouped {
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        let (rect, _) = ui.allocate_exact_size(
                                            Vec2::splat(36.0),
                                            egui::Sense::hover(),
                                        );
                                        let init = author
                                            .chars()
                                            .next()
                                            .unwrap_or('?')
                                            .to_uppercase()
                                            .next()
                                            .unwrap_or('?');
                                        ui.painter().circle_filled(rect.center(), 18.0, ACCENT_DIM);
                                        ui.painter().text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            init.to_string(),
                                            FontId::proportional(14.0),
                                            TEXT_PRIMARY,
                                        );
                                        ui.add_space(8.0);
                                        ui.label(
                                            RichText::new(author)
                                                .strong()
                                                .size(14.0)
                                                .color(TEXT_PRIMARY),
                                        );
                                    });
                                }
                                ui.horizontal_wrapped(|ui| {
                                    ui.add_space(52.0);
                                    ui.label(
                                        RichText::new(&msg.content)
                                            .size(14.0)
                                            .color(Color32::from_rgb(195, 197, 210)),
                                    );
                                });
                                if !grouped {
                                    ui.add_space(2.0);
                                }
                            }
                        }
                        ui.add_space(8.0);
                    });
            });
    }
}

// ── Widget helpers ──────────────────────────────────────────────────────────

fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(11.0).color(TEXT_DIM).strong());
    ui.add_space(4.0);
}

fn show_error(ui: &mut egui::Ui, err: &str) {
    if err.is_empty() {
        return;
    }
    egui::Frame::none()
        .fill(Color32::from_rgba_unmultiplied(240, 71, 71, 35))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.label(RichText::new(err).color(RED).size(13.0));
        });
    ui.add_space(12.0);
}

fn text_field<'a>(value: &'a mut String, password: bool) -> egui::TextEdit<'a> {
    egui::TextEdit::singleline(value)
        .password(password)
        .desired_width(f32::INFINITY)
        .font(FontId::proportional(14.0))
}

fn accent_btn_sized(ui: &mut egui::Ui, label: &str, size: Vec2) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let bg = if resp.is_pointer_button_down_on() || resp.hovered() {
            ACCENT_HOVER
        } else {
            ACCENT
        };
        ui.painter().rect_filled(rect, Rounding::same(8.0), bg);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            FontId::proportional(14.0),
            Color32::WHITE,
        );
    }
    resp
}

fn accent_btn_full(ui: &mut egui::Ui, label: &str, height: f32) -> egui::Response {
    let width = ui.available_width();
    accent_btn_sized(ui, label, Vec2::new(width, height))
}

fn icon_btn<'a>(icon: &'a str, color: Color32, tip: &'a str) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        ui.add(
            egui::Button::new(RichText::new(icon).size(16.0).color(color))
                .fill(Color32::TRANSPARENT)
                .frame(false)
                .rounding(Rounding::same(6.0)),
        )
        .on_hover_text(tip)
    }
}

fn section_header(ui: &mut egui::Ui, label: &str) {
    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(12.0, 4.0))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(11.0).color(TEXT_MUTED).strong());
        });
}

fn channel_row(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let bg = if selected {
        Color32::from_rgba_unmultiplied(88, 101, 242, 40)
    } else {
        Color32::TRANSPARENT
    };
    let color = if selected { TEXT_PRIMARY } else { TEXT_DIM };
    egui::Frame::none()
        .fill(bg)
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add(
                egui::Label::new(RichText::new(label).size(13.5).color(color))
                    .sense(egui::Sense::click()),
            )
        })
        .inner
}

// ── eframe App ──────────────────────────────────────────────────────────────

impl eframe::App for CrabCordApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMsg::LoginOk(r) | AsyncMsg::RegisterOk(r) => {
                    self.after_auth(r);
                }
                AsyncMsg::LoginErr(e) | AsyncMsg::RegisterErr(e) => {
                    self.auth_error = e;
                    self.auth_loading = false;
                }
                AsyncMsg::ServersOk(servers) => {
                    let had = !self.selected_server_id.is_empty();
                    self.servers = servers;
                    if !had {
                        if let Some(first) = self.servers.first() {
                            let sid = first.id.clone();
                            self.selected_server_id = sid.clone();
                            self.fetch_channels(&sid);
                            self.fetch_voice_channels(&sid);
                        }
                    }
                }
                AsyncMsg::ChannelsOk(ch) => {
                    self.channels = ch;
                }
                AsyncMsg::VoiceChannelsOk(vc) => {
                    self.voice_channels = vc;
                }
                AsyncMsg::MessagesOk(msgs) => {
                    self.messages = msgs;
                }
                AsyncMsg::VoiceJoinOk(payload) => {
                    if let Some((id, name)) = payload.split_once('|') {
                        self.voice_session = Some(VoiceSession {
                            channel_id: id.to_string(),
                            channel_name: name.to_string(),
                            muted: false,
                            deafened: false,
                        });
                    }
                }
                AsyncMsg::VoiceLeaveOk => {
                    self.voice_session = None;
                }
                AsyncMsg::VoiceStateOk => {}
                AsyncMsg::ApiErr(e) => {
                    eprintln!("API error: {}", e);
                }
            }
        }

        if self.view == AppView::Chat {
            if !self.selected_channel_id.is_empty()
                && self.last_poll.elapsed() > Duration::from_secs(3)
            {
                self.last_poll = Instant::now();
                self.fetch_messages();
            }
            if !self.selected_server_id.is_empty()
                && self.last_voice_poll.elapsed() > Duration::from_secs(5)
            {
                self.last_voice_poll = Instant::now();
                let sid = self.selected_server_id.clone();
                self.fetch_voice_channels(&sid);
            }
        }

        // Apply visuals before drawing. override_text_color=None lets
        // RichText colors take effect, while fg_stroke handles plain-string buttons.
        let mut v = egui::Visuals::dark();
        v.override_text_color = None;
        v.widgets.noninteractive.bg_fill = BG_INPUT;
        v.widgets.noninteractive.fg_stroke.color = TEXT_DIM;
        v.widgets.inactive.bg_fill = BG_INPUT;
        v.widgets.inactive.fg_stroke.color = TEXT_PRIMARY;
        v.widgets.hovered.bg_fill = ACCENT_HOVER;
        v.widgets.hovered.fg_stroke.color = Color32::WHITE;
        v.widgets.active.bg_fill = ACCENT;
        v.widgets.active.fg_stroke.color = Color32::WHITE;
        v.selection.bg_fill = ACCENT_DIM;
        v.panel_fill = BG_CHAT;
        v.window_fill = BG_SIDEBAR;
        v.window_rounding = Rounding::same(12.0);
        ctx.set_visuals(v);

        match self.view.clone() {
            AppView::Login => self.show_login(ctx),
            AppView::Register => self.show_register(ctx),
            AppView::Chat => self.show_chat(ctx),
        }

        if self.view == AppView::Chat || self.auth_loading {
            ctx.request_repaint_after(Duration::from_secs(3));
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    let icon_bytes = include_bytes!("../../public/icon.ico");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = image.dimensions();
    let icon = egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([700.0, 480.0])
            .with_icon(Arc::new(icon)),
        ..Default::default()
    };

    eframe::run_native(
        "CrabCord",
        options,
        Box::new(|_cc| Box::new(CrabCordApp::default())),
    )
}
