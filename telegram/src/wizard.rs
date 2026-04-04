// wizard.rs — TelegramSetupWizard (State Machine).
//
// Design Pattern: State Machine
//   Each step is a distinct enum variant; transitions are validated before
//   advancing. The wizard is driven by view.rs or the CLI in main.rs.
//
// Steps:
//   BotToken      → admin enters the secret reference for the bot token
//   AllowedChats  → optionally restrict to specific chat IDs
//   Confirm       → review + confirm before writing config
//   Done          → produces TelegramChannelConfig

use fs_channel_telegram::config::{TelegramChannelConfig, TelegramConfigStore};
use std::path::PathBuf;

// ── WizardStep ────────────────────────────────────────────────────────────────

/// One step in the Telegram setup wizard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardStep {
    /// Enter the bot token secret reference.
    BotToken,
    /// Restrict to specific chat IDs (optional, skippable).
    AllowedChats,
    /// Review all inputs before saving.
    Confirm,
    /// Wizard complete — `TelegramChannelConfig` is ready.
    Done,
}

impl WizardStep {
    /// FTL key for the step title.
    #[must_use]
    pub fn title_key(&self) -> &'static str {
        match self {
            Self::BotToken => "channel-telegram-wizard-step-token-title",
            Self::AllowedChats => "channel-telegram-wizard-step-chats-title",
            Self::Confirm => "channel-telegram-wizard-step-confirm-title",
            Self::Done => "channel-telegram-wizard-step-done-title",
        }
    }
}

// ── WizardOutcome ─────────────────────────────────────────────────────────────

/// The result produced when the wizard reaches `Done`.
#[derive(Debug, Clone)]
pub enum WizardOutcome {
    /// Config saved successfully.
    Saved(TelegramChannelConfig),
    /// User cancelled the wizard.
    Cancelled,
}

// ── TelegramSetupWizard ───────────────────────────────────────────────────────

/// State machine that collects Telegram adapter configuration step by step.
pub struct TelegramSetupWizard {
    step: WizardStep,
    config: TelegramChannelConfig,
    config_path: PathBuf,
}

impl TelegramSetupWizard {
    /// Create a new wizard writing to `config_path`.
    #[must_use]
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            step: WizardStep::BotToken,
            config: TelegramChannelConfig::default(),
            config_path,
        }
    }

    /// Return the current wizard step.
    #[must_use]
    pub fn step(&self) -> &WizardStep {
        &self.step
    }

    /// Return the current (partial) config being built.
    #[must_use]
    pub fn config(&self) -> &TelegramChannelConfig {
        &self.config
    }

    /// Set the bot token secret reference and advance to the next step.
    ///
    /// Returns `Err` if `token_ref` is empty.
    ///
    /// # Errors
    ///
    /// Returns a string error message if the token reference is empty.
    pub fn set_bot_token(&mut self, token_ref: impl Into<String>) -> Result<(), String> {
        let token_ref = token_ref.into();
        if token_ref.is_empty() {
            return Err("Bot token reference must not be empty".into());
        }
        if !token_ref.starts_with("env:") && !token_ref.starts_with("file:") {
            return Err("Bot token reference must start with 'env:' or 'file:'".into());
        }
        self.config.bot_token_ref = token_ref;
        self.step = WizardStep::AllowedChats;
        Ok(())
    }

    /// Set the allowed chat IDs (empty slice = all chats) and advance.
    pub fn set_allowed_chats(&mut self, chat_ids: Vec<i64>) {
        self.config.allowed_chat_ids = chat_ids;
        self.step = WizardStep::Confirm;
    }

    /// Confirm and save the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error string if the config cannot be written to disk.
    pub fn confirm(&mut self) -> Result<WizardOutcome, String> {
        let store = TelegramConfigStore::new(&self.config_path);
        store.save(&self.config).map_err(|e| e.to_string())?;
        self.step = WizardStep::Done;
        Ok(WizardOutcome::Saved(self.config.clone()))
    }

    /// Cancel the wizard without saving.
    #[must_use]
    pub fn cancel(mut self) -> WizardOutcome {
        self.step = WizardStep::Done;
        WizardOutcome::Cancelled
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn wizard() -> TelegramSetupWizard {
        TelegramSetupWizard::new(PathBuf::from("/tmp/test-telegram/config.toml"))
    }

    #[test]
    fn initial_step_is_bot_token() {
        assert_eq!(wizard().step(), &WizardStep::BotToken);
    }

    #[test]
    fn set_bot_token_advances_to_chats() {
        let mut w = wizard();
        w.set_bot_token("env:FS_TELEGRAM_BOT_TOKEN").unwrap();
        assert_eq!(w.step(), &WizardStep::AllowedChats);
    }

    #[test]
    fn empty_token_ref_is_rejected() {
        let mut w = wizard();
        assert!(w.set_bot_token("").is_err());
    }

    #[test]
    fn plain_text_token_is_rejected() {
        let mut w = wizard();
        assert!(w.set_bot_token("123456:ABCDEFGH").is_err());
    }

    #[test]
    fn set_chats_advances_to_confirm() {
        let mut w = wizard();
        w.set_bot_token("env:FS_TELEGRAM_BOT_TOKEN").unwrap();
        w.set_allowed_chats(vec![12345]);
        assert_eq!(w.step(), &WizardStep::Confirm);
        assert_eq!(w.config().allowed_chat_ids, vec![12345]);
    }

    #[test]
    fn empty_chats_means_all_allowed() {
        let mut w = wizard();
        w.set_bot_token("file:/run/secrets/tg").unwrap();
        w.set_allowed_chats(vec![]);
        assert!(w.config().allowed_chat_ids.is_empty());
    }

    #[test]
    fn cancel_returns_cancelled() {
        let w = wizard();
        assert!(matches!(w.cancel(), WizardOutcome::Cancelled));
    }
}
