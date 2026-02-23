use eframe::egui;
use serde::{Deserialize, Serialize};
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

#[derive(Deserialize, Debug, Clone)]
struct LoginResponse {
    token: String,
    refresh_token: Option<String>,
    user: User,
}

#[derive(Debug)]
struct LoginResult {
    token: String,
    user: User,
    channel_id: String,
}

#[derive(PartialEq)]
enum AppView {
    Login,
    Chat,
}

enum LoginState {
    Idle,
    Loading,
    Success,
    Error(String),
}

struct CrabCordApp {
    token: String,
    email_input: String,
    password_input: String,
    login_state: LoginState,
    view: AppView,
    current_user: Option<User>,

    current_channel_id: String,
    message_input: String,
    messages: Vec<Message>,
    last_poll: Instant,

    login_rx: Receiver<Result<LoginResult, String>>,
    login_tx: Sender<Result<LoginResult, String>>,

    msg_rx: Receiver<Result<Vec<Message>, String>>,
    msg_tx: Sender<Result<Vec<Message>, String>>,
}

impl Default for CrabCordApp {
    fn default() -> Self {
        let (login_tx, login_rx) = std::sync::mpsc::channel();
        let (msg_tx, msg_rx) = std::sync::mpsc::channel();
        Self {
            token: String::new(),
            email_input: String::new(),
            password_input: String::new(),
            login_state: LoginState::Idle,
            view: AppView::Login,
            current_user: None,
            current_channel_id: String::new(),
            message_input: String::new(),
            messages: Vec::new(),
            last_poll: Instant::now(),
            login_rx,
            login_tx,
            msg_rx,
            msg_tx,
        }
    }
}

impl CrabCordApp {
    fn perform_login(&mut self) {
        self.login_state = LoginState::Loading;

        let tx = self.login_tx.clone();
        let email = self.email_input.clone();
        let password = self.password_input.clone();

        tokio::spawn(async move {
            let client = reqwest::Client::new();

            let login_body = serde_json::json!({
                "username": "Test",
                "email": email,
                "password": password,
            });

            let res = client
                .post("http://localhost:3000/auth/register")
                .json(&login_body)
                .send()
                .await;

            let login_data: LoginResponse = match res {
                Ok(resp) if resp.status().is_success() => match resp.json().await {
                    Ok(d) => d,
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                        return;
                    }
                },
                Ok(resp) => {
                    let _ = tx.send(Err(format!("Login failed: {}", resp.status())));
                    return;
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    return;
                }
            };

            let token = login_data.token.clone();

            let server_res = client
                .post("http://localhost:3000/servers")
                .header("Authorization", format!("Bearer {}", token))
                .json(&serde_json::json!({ "name": "Test Server" }))
                .send()
                .await;

            let server_id = match server_res {
                Ok(resp) => {
                    let v: serde_json::Value = resp.json().await.unwrap_or_default();
                    v["id"].as_str().unwrap_or("").to_string()
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    return;
                }
            };

            if server_id.is_empty() {
                let _ = tx.send(Err("Failed to create server".to_string()));
                return;
            }

            let chan_res = client
                .post(format!(
                    "http://localhost:3000/servers/{}/channels",
                    server_id
                ))
                .header("Authorization", format!("Bearer {}", token))
                .json(&serde_json::json!({ "name": "general" }))
                .send()
                .await;

            let channel_id = match chan_res {
                Ok(resp) => {
                    let v: serde_json::Value = resp.json().await.unwrap_or_default();
                    v["id"].as_str().unwrap_or("").to_string()
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    return;
                }
            };

            if channel_id.is_empty() {
                let _ = tx.send(Err("Failed to create channel".to_string()));
                return;
            }

            let _ = tx.send(Ok(LoginResult {
                token,
                user: login_data.user,
                channel_id,
            }));
        });
    }

    fn fetch_messages(&self) {
        let tx = self.msg_tx.clone();
        let channel_id = self.current_channel_id.clone();
        let token = self.token.clone();

        if channel_id.is_empty() {
            return;
        }

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let url = format!("http://localhost:3000/channels/{}/messages", channel_id);

            let res = client
                .get(url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await;

            match res {
                Ok(resp) => {
                    let data = resp.json::<Vec<Message>>().await;
                    let _ = tx.send(data.map_err(|e| e.to_string()));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });
    }

    fn send_message(&mut self) {
        let tx = self.msg_tx.clone();
        let token = self.token.clone();
        let channel_id = self.current_channel_id.clone();
        let content = self.message_input.clone();
        self.message_input.clear();

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let url = format!("http://localhost:3000/channels/{}/messages", channel_id);

            let res = client
                .post(url)
                .header("Authorization", format!("Bearer {}", token))
                .json(&MessageRequest { content })
                .send()
                .await;

            match res {
                Ok(resp) if resp.status().is_success() => {
                    let fetch_url =
                        format!("http://localhost:3000/channels/{}/messages", channel_id);
                    if let Ok(fetch_resp) = client
                        .get(fetch_url)
                        .header("Authorization", format!("Bearer {}", token))
                        .send()
                        .await
                    {
                        let data = fetch_resp.json::<Vec<Message>>().await;
                        let _ = tx.send(data.map_err(|e| e.to_string()));
                    }
                }
                Ok(resp) => {
                    let _ = tx.send(Err(format!("Send failed: {}", resp.status())));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });
    }

    fn show_login(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.heading("CrabCord");
                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.set_min_width(300.0);
                    ui.set_max_width(300.0);

                    ui.label("Email");
                    ui.text_edit_singleline(&mut self.email_input);
                    ui.add_space(6.0);

                    ui.label("Password");
                    ui.add(egui::TextEdit::singleline(&mut self.password_input).password(true));
                    ui.add_space(12.0);

                    match &self.login_state {
                        LoginState::Loading => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Logging in...");
                            });
                        }
                        _ => {
                            if ui
                                .add_sized([300.0, 32.0], egui::Button::new("Login / Register"))
                                .clicked()
                            {
                                self.perform_login();
                            }
                        }
                    }

                    if let LoginState::Error(err) = &self.login_state {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                    }
                });
            });
        });
    }

    fn show_chat(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("input_bar")
            .min_height(48.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let msg_edit = ui.add_sized(
                        [ui.available_width() - 70.0, 32.0],
                        egui::TextEdit::singleline(&mut self.message_input).hint_text("Message..."),
                    );
                    let send = ui.add_sized([64.0, 32.0], egui::Button::new("Send"));

                    if send.clicked()
                        || (msg_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        if !self.message_input.is_empty() && !self.current_channel_id.is_empty() {
                            self.send_message();
                        }
                    }
                });
                ui.add_space(4.0);
            });

        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🦀 CrabCord");
                ui.separator();
                if let Some(user) = &self.current_user {
                    ui.label(format!("Logged in as {}", user.username));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("# {}", self.current_channel_id))
                            .weak()
                            .small(),
                    );
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let scroll = egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true);

            scroll.show(ui, |ui| {
                if self.messages.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("No messages yet.").weak());
                    });
                } else {
                    for msg in &self.messages {
                        ui.horizontal_wrapped(|ui| {
                            let name = msg
                                .username
                                .clone()
                                .unwrap_or_else(|| "Unknown".to_string());
                            ui.label(egui::RichText::new(format!("{}:", name)).strong());
                            ui.label(&msg.content);
                        });
                        ui.add_space(2.0);
                    }
                }
            });
        });
    }
}

impl eframe::App for CrabCordApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(result) = self.login_rx.try_recv() {
            match result {
                Ok(res) => {
                    self.token = res.token;
                    self.current_channel_id = res.channel_id;
                    self.current_user = Some(res.user);
                    self.login_state = LoginState::Success;
                    self.view = AppView::Chat;
                    self.fetch_messages();
                }
                Err(e) => self.login_state = LoginState::Error(e),
            }
        }

        while let Ok(msg_res) = self.msg_rx.try_recv() {
            match msg_res {
                Ok(msgs) => self.messages = msgs,
                Err(e) => eprintln!("Message error: {}", e),
            }
        }

        if self.view == AppView::Chat && self.last_poll.elapsed() > Duration::from_secs(3) {
            self.last_poll = Instant::now();
            self.fetch_messages();
        }

        match self.view {
            AppView::Login => self.show_login(ctx),
            AppView::Chat => self.show_chat(ctx),
        }

        if matches!(self.view, AppView::Chat) || matches!(self.login_state, LoginState::Loading) {
            ctx.request_repaint_after(Duration::from_secs(3));
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "CrabCord",
        options,
        Box::new(|_cc| Box::new(CrabCordApp::default())),
    )
}
