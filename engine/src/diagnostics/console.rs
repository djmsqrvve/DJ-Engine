use crate::story_graph::GraphExecutor;
use bevy::{
    app::AppExit,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use std::{
    io::{self, IsTerminal, Write},
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
};

/// Resource holding the receiver for console input.
#[derive(Resource)]
struct ConsoleReceiver(Arc<Mutex<Receiver<String>>>);

/// Resource storing log messages for the Editor UI.
#[derive(Resource, Default, Clone)]
pub struct ConsoleLogStore {
    pub logs: Vec<String>,
}

impl ConsoleLogStore {
    pub fn log(&mut self, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        self.logs.push(format!("[{}] {}", timestamp, message));
        // Keep only last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
}

/// Event fired when a CLI command is entered.
#[derive(Message)]
pub struct ConsoleCommandEvent(pub String);

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        let interactive_console = io::stdin().is_terminal();

        if interactive_console {
            let (tx, rx) = mpsc::channel();

            // Spawn background thread for stdin only when the process is interactive.
            std::thread::spawn(move || {
                let stdin = io::stdin();
                let mut input = String::new();
                loop {
                    print!("dj> ");
                    let _ = io::stdout().flush();

                    input.clear();
                    match stdin.read_line(&mut input) {
                        Ok(0) => break,
                        Ok(_) => {
                            let cmd = input.trim().to_string();
                            if !cmd.is_empty() && tx.send(cmd).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            app.insert_resource(ConsoleReceiver(Arc::new(Mutex::new(rx))));
        } else {
            debug!("Console CLI disabled: stdin is not interactive.");
        }

        app.init_resource::<ConsoleLogStore>()
            .add_message::<ConsoleCommandEvent>()
            .add_systems(
                Update,
                listen_for_console_input.run_if(resource_exists::<ConsoleReceiver>),
            )
            .add_systems(Update, handle_console_commands);

        if interactive_console {
            info!("Console CLI API initialized. Type 'help' in terminal for commands.");
        }
    }
}

fn listen_for_console_input(
    receiver: Res<ConsoleReceiver>,
    mut events: MessageWriter<ConsoleCommandEvent>,
) {
    if let Ok(rx) = receiver.0.lock() {
        while let Ok(cmd) = rx.try_recv() {
            events.write(ConsoleCommandEvent(cmd));
        }
    }
}

fn handle_console_commands(
    mut events: MessageReader<ConsoleCommandEvent>,
    mut app_exit: MessageWriter<AppExit>,
    windows: Query<(Entity, &Window)>,
    entities: Query<Entity>,
    executor: Option<Res<GraphExecutor>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    for event in events.read() {
        let cmd = event.0.to_lowercase();
        let args: Vec<&str> = cmd.split_whitespace().collect();

        if args.is_empty() {
            continue;
        }

        match args[0] {
            "help" => {
                println!("\n--- DJ Engine CLI Help ---");
                println!("help     - Show this help");
                println!("windows  - List open windows and status");
                println!("entities - Show entity count breakdown");
                println!("story    - Show story graph execution state");
                println!("fps      - Show current performance metrics");
                println!("exit     - Close the engine");
                println!("--------------------------\n");
            }
            "windows" => {
                println!("\n--- Active Windows ---");
                let mut found = false;
                for (entity, window) in windows.iter() {
                    found = true;
                    println!(
                        "Entity: {:?}\nTitle: \"{}\"\nResolution: {}x{}\nScale Factor: {}\nPosition: {:?}\nFocused: {}\nVisible: {}",
                        entity, window.title, window.width(), window.height(),
                        window.scale_factor(), window.position, window.focused, window.visible
                    );
                }
                if !found {
                    println!("No active windows detected (Headless mode?)");
                }
                println!("----------------------\n");
            }
            "entities" => {
                let count = entities.iter().count();
                println!("\nTotal Entities: {}\n", count);
            }
            "story" => {
                println!("\n--- Story Graph Status ---");
                if let Some(exec) = executor.as_ref() {
                    println!("Status: {:?}", exec.status);
                    println!("Active Node: {:?}", exec.current_node);
                    println!("Has Active Graph: {}", exec.active_graph.is_some());
                } else {
                    println!("No GraphExecutor resource found.");
                }
                println!("--------------------------\n");
            }
            "fps" => {
                let fps = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|diag| diag.smoothed())
                    .unwrap_or(0.0);
                let ft = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
                    .and_then(|diag| diag.smoothed())
                    .unwrap_or(0.0);
                println!("\nFPS: {:.1}\nFrame Time: {:.2}ms\n", fps, ft);
            }
            "exit" | "quit" => {
                println!("Exiting engine...");
                app_exit.write(AppExit::Success);
            }
            _ => {
                println!(
                    "Unknown command: '{}'. Type 'help' for available commands.",
                    args[0]
                );
            }
        }
    }
}
