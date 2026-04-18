use anyhow::Context;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

const IBRON_SH: &str = include_str!("../../assets/shell-integration/ibron.sh");
const IBRON_PS1: &str = include_str!("../../assets/shell-integration/ibron.ps1");
const IBRON_FISH: &str = include_str!("../../assets/shell-integration/ibron.fish");

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum IntegrationShell {
    Bash,
    Zsh,
    Powershell,
    Fish,
}

#[derive(Debug, Parser, Clone)]
pub struct ShellIntegrationCommand {
    /// Target shell. Defaults to auto-detecting from $SHELL on Unix,
    /// PowerShell on Windows.
    #[arg(long, value_enum)]
    pub shell: Option<IntegrationShell>,

    /// Print the script to stdout. This is the default when neither
    /// --print nor --install is given.
    #[arg(long, conflicts_with = "install")]
    pub print: bool,

    /// Write the script to a canonical location under the user's config
    /// directory and print the one-line snippet to source it from the
    /// shell profile.
    #[arg(long)]
    pub install: bool,
}

impl ShellIntegrationCommand {
    pub fn run(self) -> anyhow::Result<()> {
        let shell = self.shell.unwrap_or_else(detect_shell);
        let (script, install_path, profile_snippet) = layout_for(shell)?;

        if self.install {
            std::fs::create_dir_all(
                install_path
                    .parent()
                    .context("install path has no parent directory")?,
            )?;
            std::fs::write(&install_path, script)?;
            eprintln!("Wrote {}", install_path.display());
            eprintln!();
            eprintln!("Add this line to your shell profile:");
            println!("{}", profile_snippet);
        } else {
            print!("{}", script);
        }
        Ok(())
    }
}

fn layout_for(shell: IntegrationShell) -> anyhow::Result<(&'static str, PathBuf, String)> {
    let config = dirs_next::config_dir().context("no config dir for current user")?;
    let base = config.join("ibron").join("shell-integration");
    Ok(match shell {
        IntegrationShell::Bash | IntegrationShell::Zsh => {
            let path = base.join("ibron.sh");
            let snippet = format!("source \"{}\"", path.display());
            (IBRON_SH, path, snippet)
        }
        IntegrationShell::Powershell => {
            let path = base.join("ibron.ps1");
            let snippet = format!(". \"{}\"", path.display());
            (IBRON_PS1, path, snippet)
        }
        IntegrationShell::Fish => {
            let path = base.join("ibron.fish");
            let snippet = format!("source \"{}\"", path.display());
            (IBRON_FISH, path, snippet)
        }
    })
}

fn detect_shell() -> IntegrationShell {
    #[cfg(windows)]
    {
        IntegrationShell::Powershell
    }
    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_default();
        if shell.contains("fish") {
            IntegrationShell::Fish
        } else if shell.contains("zsh") {
            IntegrationShell::Zsh
        } else {
            IntegrationShell::Bash
        }
    }
}
