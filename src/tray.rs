use std::sync::mpsc;

use ksni::{Status, TrayMethods};
use tracing::warn;

pub enum TrayMessage {
    OpenWindow,
    GrabScreenshot,
    OpenFile,
    Quit,
}

struct FrogTray {
    sender: mpsc::Sender<TrayMessage>,
}

impl ksni::Tray for FrogTray {
    fn id(&self) -> String {
        "com.tenderowl.frog".into()
    }

    fn icon_name(&self) -> String {
        "com.tenderowl.frog".into()
    }

    fn title(&self) -> String {
        "Frog".into()
    }

    fn status(&self) -> Status {
        Status::Active
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(TrayMessage::OpenWindow);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Open Window".into(),
                icon_name: "window-new".into(),
                activate: Box::new(|this: &mut FrogTray| {
                    let _ = this.sender.send(TrayMessage::OpenWindow);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Grab Screenshot".into(),
                icon_name: "camera-photo".into(),
                activate: Box::new(|this: &mut FrogTray| {
                    let _ = this.sender.send(TrayMessage::GrabScreenshot);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Open File".into(),
                icon_name: "document-open".into(),
                activate: Box::new(|this: &mut FrogTray| {
                    let _ = this.sender.send(TrayMessage::OpenFile);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|this: &mut FrogTray| {
                    let _ = this.sender.send(TrayMessage::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn spawn_tray(sender: mpsc::Sender<TrayMessage>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(async {
            let tray = FrogTray { sender };
            match tray.spawn().await {
                Ok(_handle) => {
                    // Keep the tray alive
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                    }
                }
                Err(e) => {
                    warn!("Failed to spawn tray icon (no system tray available): {e}");
                }
            }
        });
    });
}
