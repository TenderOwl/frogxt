use std::sync::OnceLock;

use gtk::gio;
use gtk::prelude::*;

use crate::config::APP_ID;

const POSTHOG_API_KEY: &str = "phc_HpETCN6yQKZIr8gr6mBQTd3H0SjKUBrNMI3AizoX97f";
const POSTHOG_HOST: &str = "https://eu.posthog.com";

static CLIENT: OnceLock<posthog_rs::Client> = OnceLock::new();

fn ensure_client() -> Option<&'static posthog_rs::Client> {
    if let Some(client) = CLIENT.get() {
        return Some(client);
    }

    let options = posthog_rs::ClientOptions::from((POSTHOG_API_KEY, POSTHOG_HOST));
    let client = posthog_rs::client(options);
    CLIENT.set(client).ok();
    CLIENT.get()
}

fn ensure_installation_id() -> String {
    let settings = gio::Settings::new(APP_ID);
    let installation_id = settings.string("installation-id").to_string();

    if installation_id.is_empty() {
        let new_id = nanoid::nanoid!();
        settings.set_string("installation-id", &new_id).unwrap();
        tracing::info!("Generated new installation ID: {}", new_id);
        new_id
    } else {
        installation_id
    }
}

pub fn init() {
    ensure_installation_id();
}

pub fn flush() {
    if let Some(client) = CLIENT.get() {
        client.flush();
    }
}

pub fn shutdown() {
    if let Some(client) = CLIENT.get() {
        client.shutdown();
    }
}

fn is_enabled() -> bool {
    let settings = gio::Settings::new(APP_ID);
    let is_active = settings.boolean("telemetry");
    let installation_id = settings.string("installation-id").to_string();
    is_active && !installation_id.is_empty()
}

fn distinct_id() -> String {
    let settings = gio::Settings::new(APP_ID);
    settings.string("installation-id").to_string()
}

pub fn capture(event: &str) {
    capture_with_props(event, &[]);
}

pub fn capture_with_props(event: &str, props: &[(&str, &dyn ToPostHogProp)]) {
    if !is_enabled() {
        return;
    }
    let Some(client) = ensure_client() else {
        return;
    };

    let mut eh = posthog_rs::Event::new(event, &distinct_id());
    for (key, val) in props {
        val.insert_prop(&mut eh, key);
    }

    client.capture(eh);
}

pub fn capture_page_view(page_name: &str) {
    capture_with_props("$pageview", &[("$current_url", &page_name)]);
}

// -- Property trait --

pub trait ToPostHogProp {
    fn insert_prop(&self, event: &mut posthog_rs::Event, key: &str);
}

impl ToPostHogProp for str {
    fn insert_prop(&self, event: &mut posthog_rs::Event, key: &str) {
        let _ = event.insert_prop(key, self);
    }
}

impl ToPostHogProp for &str {
    fn insert_prop(&self, event: &mut posthog_rs::Event, key: &str) {
        let _ = event.insert_prop(key, *self);
    }
}

impl ToPostHogProp for String {
    fn insert_prop(&self, event: &mut posthog_rs::Event, key: &str) {
        let _ = event.insert_prop(key, self.as_str());
    }
}

impl ToPostHogProp for bool {
    fn insert_prop(&self, event: &mut posthog_rs::Event, key: &str) {
        let _ = event.insert_prop(key, *self);
    }
}

impl ToPostHogProp for i32 {
    fn insert_prop(&self, event: &mut posthog_rs::Event, key: &str) {
        let _ = event.insert_prop(key, *self);
    }
}

impl ToPostHogProp for f64 {
    fn insert_prop(&self, event: &mut posthog_rs::Event, key: &str) {
        let _ = event.insert_prop(key, *self);
    }
}
