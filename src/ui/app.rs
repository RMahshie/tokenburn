use std::time::{Duration, Instant};

use crate::data::{DashboardData, TimeRange, Tool};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeView {
    Main,
    Subagents,
    Combined,
    UsageCache,
}

impl ClaudeView {
    pub fn label(self) -> &'static str {
        match self {
            ClaudeView::Main => "CLAUDE CODE",
            ClaudeView::Subagents => "CLAUDE SUBAGENTS",
            ClaudeView::Combined => "CLAUDE ALL",
            ClaudeView::UsageCache => "CLAUDE USAGE CACHE",
        }
    }

    fn next(self) -> Self {
        match self {
            ClaudeView::Main => ClaudeView::Subagents,
            ClaudeView::Subagents => ClaudeView::Combined,
            ClaudeView::Combined => ClaudeView::UsageCache,
            ClaudeView::UsageCache => ClaudeView::Main,
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub data: DashboardData,
    pub range: TimeRange,
    pub paused: bool,
    pub quit: bool,
    pub show_help: bool,
    pub claude_view: ClaudeView,
    pub active_tab: usize,
    pub interval: Duration,
    pub last_refresh: Instant,
}

impl App {
    pub fn new(data: DashboardData, range: TimeRange, interval_secs: u64) -> Self {
        Self {
            data,
            range,
            paused: false,
            quit: false,
            show_help: false,
            claude_view: ClaudeView::Main,
            active_tab: 0,
            interval: Duration::from_secs(interval_secs.max(1)),
            last_refresh: Instant::now(),
        }
    }

    pub fn tab_count(&self) -> usize {
        self.data.summaries.len()
    }

    pub fn next_tab(&mut self) {
        let count = self.tab_count();
        if count > 0 {
            self.active_tab = (self.active_tab + 1) % count;
            self.reset_claude_view_if_needed();
        }
    }

    pub fn prev_tab(&mut self) {
        let count = self.tab_count();
        if count > 0 {
            self.active_tab = (self.active_tab + count - 1) % count;
            self.reset_claude_view_if_needed();
        }
    }

    pub fn cycle_range(&mut self) {
        self.range = match self.range {
            TimeRange::LastHours(24) => TimeRange::LastDays(7),
            TimeRange::LastDays(7) => TimeRange::LastDays(30),
            TimeRange::LastDays(30) => TimeRange::Lifetime,
            _ => TimeRange::LastHours(24),
        };
        self.last_refresh = Instant::now() - self.interval;
    }

    pub fn cycle_claude_view(&mut self) {
        if self.active_tool() == Some(Tool::Claude) {
            self.claude_view = self.claude_view.next();
        }
    }

    pub fn refresh_due(&self) -> bool {
        !self.paused && self.last_refresh.elapsed() >= self.interval
    }

    pub fn active_tool(&self) -> Option<Tool> {
        self.data
            .summaries
            .get(self.active_tab)
            .map(|summary| summary.tool)
    }

    fn reset_claude_view_if_needed(&mut self) {
        if self.active_tool() != Some(Tool::Claude) {
            self.claude_view = ClaudeView::Main;
        }
    }

    pub fn can_apply_claude_retention(&self) -> bool {
        self.active_tool() == Some(Tool::Claude)
            && self
                .data
                .claude_retention
                .as_ref()
                .is_some_and(|status| status.needs_update)
    }
}
