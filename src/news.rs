use bevy::prelude::{Color, Resource};

use crate::network_commands::{CommandPayload, NewsEventPacket};

pub(crate) const NEWS_LIFETIME_SECONDS: f32 = 17.0;
pub(crate) const NEWS_FADE_SECONDS: f32 = 5.0;
pub(crate) const NEWS_MAX_HISTORY: usize = 50;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NewsColor {
    r: u8,
    g: u8,
    b: u8,
}

impl NewsColor {
    pub(crate) fn from_source_rgb(r: u8, g: u8, b: u8) -> Self {
        if r == 0 && g == 0 && b == 0 {
            Self { r: 1, g, b }
        } else {
            Self { r, g, b }
        }
    }

    pub(crate) fn to_bevy(self, alpha: f32) -> Color {
        Color::srgba_u8(
            self.r,
            self.g,
            self.b,
            (alpha.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NewsEntry {
    message: String,
    color: NewsColor,
    remaining_seconds: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NewsDisplayEntry<'a> {
    pub(crate) message: &'a str,
    pub(crate) color: NewsColor,
    pub(crate) alpha: f32,
}

#[derive(Clone, Debug, Default, Resource)]
pub(crate) struct NewsLog {
    entries: Vec<NewsEntry>,
    show_chat_history: bool,
}

impl NewsLog {
    pub(crate) fn toggle_chat_history(&mut self) {
        self.show_chat_history = !self.show_chat_history;
    }

    pub(crate) fn relay_source_news(
        &mut self,
        message: impl Into<String>,
        r: u8,
        g: u8,
        b: u8,
    ) -> bool {
        let Some(packet) = NewsEventPacket::new(message, r, g, b) else {
            return false;
        };
        let wire_packet = packet.encode_packet();
        let Some(payload) = wire_packet.get(8..) else {
            return false;
        };
        let Some(decoded_packet) = NewsEventPacket::decode_payload(payload) else {
            return false;
        };
        self.apply_news_event(decoded_packet)
    }

    pub(crate) fn apply_news_event(&mut self, packet: NewsEventPacket) -> bool {
        self.push_source_news(packet.message, packet.r, packet.g, packet.b);
        true
    }

    pub(crate) fn push_source_news(&mut self, message: impl Into<String>, r: u8, g: u8, b: u8) {
        self.entries.insert(
            0,
            NewsEntry {
                message: message.into(),
                color: NewsColor::from_source_rgb(r, g, b),
                remaining_seconds: NEWS_LIFETIME_SECONDS,
            },
        );
        self.entries.truncate(NEWS_MAX_HISTORY);
    }

    pub(crate) fn advance(&mut self, delta_seconds: f32) {
        let delta_seconds = delta_seconds.max(0.0);
        for entry in &mut self.entries {
            entry.remaining_seconds = (entry.remaining_seconds - delta_seconds).max(0.0);
        }
    }

    pub(crate) fn display_entry(&self, slot: usize) -> Option<NewsDisplayEntry<'_>> {
        self.entries
            .iter()
            .filter(|entry| self.show_chat_history || entry.remaining_seconds > 0.0)
            .nth(slot)
            .map(|entry| NewsDisplayEntry {
                message: &entry.message,
                color: entry.color,
                alpha: if self.show_chat_history {
                    1.0
                } else {
                    news_entry_alpha(entry.remaining_seconds)
                },
            })
    }
}

fn news_entry_alpha(remaining_seconds: f32) -> f32 {
    if remaining_seconds >= NEWS_FADE_SECONDS {
        1.0
    } else {
        (remaining_seconds / NEWS_FADE_SECONDS).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_black_news_uses_nonzero_red_like_original() {
        assert_eq!(
            NewsColor::from_source_rgb(0, 0, 0),
            NewsColor { r: 1, g: 0, b: 0 }
        );
        assert_eq!(
            NewsColor::from_source_rgb(7, 8, 9),
            NewsColor { r: 7, g: 8, b: 9 }
        );
    }

    #[test]
    fn news_log_pushes_newest_first_and_caps_history() {
        let mut log = NewsLog::default();

        for index in 0..(NEWS_MAX_HISTORY + 2) {
            assert!(log.relay_source_news(format!("message {index}"), 255, 255, 255));
        }

        assert_eq!(log.entries.len(), NEWS_MAX_HISTORY);
        assert_eq!(
            log.display_entry(0).map(|entry| entry.message),
            Some("message 51")
        );
        assert_eq!(
            log.display_entry(NEWS_MAX_HISTORY - 1)
                .map(|entry| entry.message),
            Some("message 2")
        );
        assert_eq!(log.display_entry(NEWS_MAX_HISTORY), None);
    }

    #[test]
    fn news_log_hides_expired_entries_and_fades_last_five_seconds() {
        let mut log = NewsLog::default();

        assert!(log.relay_source_news("first", 255, 255, 255));
        log.advance(12.0);
        assert_eq!(log.display_entry(0).map(|entry| entry.alpha), Some(1.0));

        log.advance(2.5);
        assert_eq!(log.display_entry(0).map(|entry| entry.alpha), Some(0.5));

        log.advance(2.5);
        assert_eq!(log.display_entry(0), None);
    }

    #[test]
    fn chat_history_toggle_shows_expired_entries_without_fade() {
        let mut log = NewsLog::default();

        assert!(log.relay_source_news("old", 255, 255, 255));
        log.advance(NEWS_LIFETIME_SECONDS);
        assert_eq!(log.display_entry(0), None);

        log.toggle_chat_history();
        assert_eq!(log.display_entry(0).map(|entry| entry.message), Some("old"));
        assert_eq!(log.display_entry(0).map(|entry| entry.alpha), Some(1.0));
    }

    #[test]
    fn news_log_rejects_source_client_too_short_messages() {
        let mut log = NewsLog::default();

        assert!(!log.relay_source_news("", 255, 255, 255));
        assert!(!log.relay_source_news("x", 255, 255, 255));
        assert_eq!(log.display_entry(0), None);

        assert!(log.relay_source_news("ok", 255, 255, 255));
        assert_eq!(log.display_entry(0).map(|entry| entry.message), Some("ok"));
    }
}
