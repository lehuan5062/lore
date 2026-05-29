// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use anstyle::AnsiColor;
use anstyle::Color;
use anstyle::Style;
use lore::interface::LoreFileAction;
use lore::interface::LoreLogLevel;

pub struct CommonStyles;
pub struct FileActionStyle;
pub struct BranchStyles;
pub struct FileDiffStyles;
pub struct LogStyles;
pub struct BisectStyles;

const FG_GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
const FG_YELLOW: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
const FG_RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
const FG_CYAN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));
const FG_GRAY: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::White)));
const FG_WHITE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightWhite)));

const FG_BOLD_RED: Style = FG_RED.bold();
const FG_BOLD_YELLOW: Style = FG_YELLOW.bold();
const FG_BOLD_GREEN: Style = FG_GREEN.bold();

const FG_BOLD_UL_GREEN: Style = FG_GREEN.underline().bold();
const FG_BOLD_UL_YELLOW: Style = FG_YELLOW.underline().bold();

const BG_GREEN: Style = Style::new().bg_color(Some(Color::Ansi(AnsiColor::Green)));
const BG_YELLOW: Style = Style::new().bg_color(Some(Color::Ansi(AnsiColor::Yellow)));
const BG_RED: Style = Style::new().bg_color(Some(Color::Ansi(AnsiColor::Red)));

const NO_STYLE: Style = Style::new();
const BOLD: Style = Style::new().bold();
const ITALIC: Style = Style::new().italic();
const BOLD_ITALIC: Style = Style::new().bold().italic();

impl CommonStyles {
    pub const LINK: Style = BOLD;
    pub const DEFAULT: Style = NO_STYLE;
    pub const HEADERS: Style = FG_CYAN;
    pub const SUCCESS: Style = FG_GREEN;
    pub const FAILURE: Style = FG_RED;
    pub const MAINTENANCE: Style = FG_BOLD_YELLOW;
    pub const SEARCH_HIGHLIGHT: Style = FG_BOLD_YELLOW;
    pub const PROGRESS_FG: Style = FG_WHITE;
    pub const PROGRESS_BG: Style = FG_GRAY;
}

impl LogStyles {
    pub const ERROR: Style = FG_BOLD_RED;
    pub const WARNING: Style = FG_YELLOW;

    pub fn from_level(level: LoreLogLevel) -> Style {
        match level {
            LoreLogLevel::Error => Self::ERROR,
            LoreLogLevel::Warn => Self::WARNING,
            _ => Style::new(),
        }
    }
}

impl FileActionStyle {
    pub const ADDED: Style = FG_GREEN;
    pub const DELETED: Style = FG_RED;
    pub const MODIFIED: Style = FG_YELLOW;

    pub const BG_ADDED: Style = BG_GREEN;
    pub const BG_DELETED: Style = BG_RED;
    pub const BG_MODIFIED: Style = BG_YELLOW;
    pub const CONFLICT: Style = BG_RED;

    pub fn from_action(action: LoreFileAction) -> Style {
        match action {
            LoreFileAction::Add => Self::ADDED,
            LoreFileAction::Delete => Self::DELETED,
            LoreFileAction::Keep | LoreFileAction::Move | LoreFileAction::Copy => Self::MODIFIED,
        }
    }

    pub fn from_action_bg(action: LoreFileAction) -> Style {
        match action {
            LoreFileAction::Add => Self::BG_ADDED,
            LoreFileAction::Delete => Self::BG_DELETED,
            LoreFileAction::Keep | LoreFileAction::Move | LoreFileAction::Copy => Self::BG_MODIFIED,
        }
    }
}

impl BranchStyles {
    pub const CURRENT_BRANCH: Style = FG_GREEN;
    pub const CONFLICT: Style = FG_RED;
    pub const DELETED: Style = FG_RED;
}

impl FileDiffStyles {
    pub const ADDITIONS: Style = FG_GREEN;
    pub const DELETIONS: Style = FG_RED;
    pub const DEFAULT: Style = CommonStyles::DEFAULT;
}

impl BisectStyles {
    pub const COMMAND: Style = ITALIC;
    pub const REVISION: Style = CommonStyles::SEARCH_HIGHLIGHT;
    pub const EMPHASIS: Style = BOLD_ITALIC;
    pub const STEP_SUCCESS: Style = FG_GREEN;
    pub const SUCCESS: Style = FG_BOLD_GREEN;
}

pub fn cli_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(FG_BOLD_UL_YELLOW)
        .header(FG_BOLD_UL_YELLOW)
        .literal(FG_GREEN)
        .invalid(FG_BOLD_RED)
        .error(FG_BOLD_RED)
        .valid(FG_BOLD_UL_GREEN)
        .placeholder(FG_GRAY)
}
