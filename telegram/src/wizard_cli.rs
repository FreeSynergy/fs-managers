// wizard_cli.rs — Interactive CLI driver for TelegramSetupWizard.
//
// Reads user input from stdin and drives the TelegramSetupWizard state machine.

use std::io::{self, BufRead, Write};
use std::path::Path;

use anyhow::Result;
use fs_channel_telegram::keys;
use fs_manager_telegram::wizard::{TelegramSetupWizard, WizardOutcome, WizardStep};

/// Run the setup wizard interactively, reading from stdin / writing to stdout.
///
/// # Errors
///
/// Returns an error if reading stdin fails or the config cannot be saved.
pub fn run_wizard(config_path: &Path) -> Result<()> {
    let stdin = io::stdin();
    let mut wizard = TelegramSetupWizard::new(config_path.to_path_buf());

    println!();
    println!("=== {} ===", fs_i18n::t(keys::WIZARD_TITLE));
    println!();

    loop {
        match wizard.step() {
            WizardStep::BotToken => {
                println!("{}", fs_i18n::t(keys::WIZARD_TOKEN_HINT));
                println!();
                print!("{} ", fs_i18n::t(keys::WIZARD_TOKEN_PROMPT));
                io::stdout().flush()?;

                let mut line = String::new();
                stdin.lock().read_line(&mut line)?;
                let input = line.trim().to_owned();

                if input.is_empty() {
                    println!("Cancelled.");
                    handle_outcome(&wizard.cancel());
                    return Ok(());
                }

                match wizard.set_bot_token(input) {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error: {e}");
                    }
                }
            }

            WizardStep::AllowedChats => {
                println!();
                println!("{}", fs_i18n::t(keys::WIZARD_CHATS_PROMPT));
                print!("> ");
                io::stdout().flush()?;

                let mut line = String::new();
                stdin.lock().read_line(&mut line)?;
                let input = line.trim().to_owned();

                let chat_ids: Vec<i64> = if input.is_empty() {
                    vec![]
                } else {
                    let mut ids = Vec::new();
                    let mut valid = true;
                    for part in input.split(',') {
                        if let Ok(id) = part.trim().parse::<i64>() {
                            ids.push(id);
                        } else {
                            eprintln!("Invalid chat ID: '{}'", part.trim());
                            valid = false;
                            break;
                        }
                    }
                    if !valid {
                        continue;
                    }
                    ids
                };

                wizard.set_allowed_chats(chat_ids);
            }

            WizardStep::Confirm => {
                println!();
                println!("{}", fs_i18n::t(keys::WIZARD_CONFIRM));
                println!(
                    "  {}: {}",
                    fs_i18n::t(keys::CONFIG_TOKEN_REF_LABEL),
                    wizard.config().bot_token_ref
                );
                let chats = if wizard.config().allowed_chat_ids.is_empty() {
                    fs_i18n::t(keys::CONFIG_CHATS_ALL).to_string()
                } else {
                    wizard
                        .config()
                        .allowed_chat_ids
                        .iter()
                        .map(i64::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                println!("  {}: {}", fs_i18n::t(keys::CONFIG_CHATS_LABEL), chats);
                println!();
                print!("Save? [Y/n] ");
                io::stdout().flush()?;

                let mut line = String::new();
                stdin.lock().read_line(&mut line)?;
                let input = line.trim().to_lowercase();

                if input == "n" || input == "no" {
                    println!("Cancelled.");
                    handle_outcome(&wizard.cancel());
                    return Ok(());
                }

                match wizard.confirm() {
                    Ok(outcome) => {
                        handle_outcome(&outcome);
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Error saving config: {e}");
                        return Err(anyhow::anyhow!(e));
                    }
                }
            }

            WizardStep::Done => {
                break;
            }
        }
    }

    Ok(())
}

fn handle_outcome(outcome: &WizardOutcome) {
    if matches!(outcome, WizardOutcome::Saved(_)) {
        println!("{}", fs_i18n::t(keys::WIZARD_SAVED));
    }
}
