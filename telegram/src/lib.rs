#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::missing_errors_doc)]
// fs-manager-telegram — Telegram channel adapter setup wizard + status manager.
//
// Design Pattern: State Machine (TelegramSetupWizard with WizardStep)
//
// Responsibilities:
//   - Guide the admin through initial Telegram bot configuration after install
//   - Steps: BotToken → AllowedChats → Confirm → Done
//   - Produces TelegramChannelConfig written via TelegramConfigStore (fs-config)
//   - View wired to fs-render (view.rs only)
//
// # Modules
//
//   - [`wizard`] — TelegramSetupWizard state machine + WizardStep
//   - [`view`]   — FsView + ManagerLayout impl (only file importing fs-render)

pub mod view;
pub mod wizard;

pub use wizard::{TelegramSetupWizard, WizardOutcome, WizardStep};
