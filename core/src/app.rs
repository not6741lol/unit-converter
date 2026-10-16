//! UI-agnostic application state.
//!
//! Every front-end (`egui`, `fltk`, `win32`, `web`) drives one of these and only
//! has to *render* it and forward user actions. Keeping the behaviour here means
//! all four behave identically and are tested once.

use crate::{ALL_CATEGORIES, Category, Mode, NumFormat, convert, fmt_with, matches, math};

/// A saved conversion — used for both the history and the favourites list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub mode: Mode,
    pub category: Category,
    pub from: usize,
    pub to: usize,
    pub input: String,
}

impl Entry {
    /// A short human-readable description, e.g. `1 Meter -> Kilometer`.
    pub fn describe(&self) -> String {
        match self.mode {
            Mode::Calculator => self.input.clone(),
            Mode::Convert => {
                let units = self.category.units();
                let from = units.get(self.from).copied().unwrap_or("?");
                let to = units.get(self.to).copied().unwrap_or("?");
                format!("{} {} -> {}", self.input, from, to)
            }
        }
    }
}

/// Everything the UI needs to draw and react to.
pub struct App {
    pub mode: Mode,
    pub category: Category,
    pub from: usize,
    pub to: usize,
    pub input: String,
    pub format: NumFormat,
    pub search: String,
    pub dark: bool,
    history: Vec<Entry>,
    favorites: Vec<Entry>,
}

/// How many history / favourite rows to keep.
const MAX_SAVED: usize = 40;

impl Default for App {
    fn default() -> Self {
        Self {
            mode: Mode::Convert,
            category: Category::Length,
            from: 0,
            to: 1,
            input: "1".to_string(),
            format: NumFormat::Auto,
            search: String::new(),
            dark: true,
            history: Vec::new(),
            favorites: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Units of the current category (clamped indices in mind).
    pub fn units(&self) -> &'static [&'static str] {
        self.category.units()
    }

    pub fn from_unit(&self) -> &'static str {
        let units = self.units();
        units.get(self.from).copied().unwrap_or("?")
    }

    pub fn to_unit(&self) -> &'static str {
        let units = self.units();
        units.get(self.to).copied().unwrap_or("?")
    }

    /// Categories matching the current search text (all of them when empty).
    pub fn filtered_categories(&self) -> Vec<Category> {
        ALL_CATEGORIES
            .iter()
            .copied()
            .filter(|c| matches(c.label(), &self.search))
            .collect()
    }

    /// Switch mode, keeping the input sensible.
    pub fn set_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.mode = mode;
            if mode == Mode::Calculator {
                self.input = "1 + 1".to_string();
            } else if self.input.trim().is_empty() {
                self.input = "1".to_string();
            }
        }
    }

    /// Switch category, resetting the unit choices.
    pub fn set_category(&mut self, category: Category) {
        if self.category != category {
            self.category = category;
            self.from = 0;
            self.to = 1.min(category.units().len().saturating_sub(1));
        }
    }

    pub fn set_from(&mut self, index: usize) {
        self.from = index.min(self.units().len().saturating_sub(1));
    }

    pub fn set_to(&mut self, index: usize) {
        self.to = index.min(self.units().len().saturating_sub(1));
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);
    }

    /// The current result as display text. Empty when there is nothing to show.
    pub fn result_text(&self) -> String {
        if self.input.trim().is_empty() {
            return String::new();
        }
        match math::eval(&self.input) {
            Ok(value) => match self.mode {
                Mode::Calculator => fmt_with(value, self.format),
                Mode::Convert => {
                    let converted = convert(self.category, self.from_unit(), self.to_unit(), value);
                    format!("{} {}", fmt_with(converted, self.format), self.to_unit())
                }
            },
            Err(_) => "(invalid expression)".to_string(),
        }
    }

    /// The *numeric* result, if the input parses.
    pub fn result_value(&self) -> Option<f64> {
        math::eval(&self.input).ok()
    }

    pub fn is_valid(&self) -> bool {
        !self.input.trim().is_empty() && self.result_value().is_some()
    }

    /// Snapshot of the current state, for saving.
    pub fn entry(&self) -> Entry {
        Entry {
            mode: self.mode,
            category: self.category,
            from: self.from,
            to: self.to,
            input: self.input.clone(),
        }
    }

    pub fn history(&self) -> &[Entry] {
        &self.history
    }

    pub fn favorites(&self) -> &[Entry] {
        &self.favorites
    }

    /// Save the current conversion to the history (most recent first).
    pub fn save(&mut self) {
        if !self.is_valid() {
            return;
        }
        let entry = self.entry();
        self.history.retain(|e| e != &entry);
        self.history.insert(0, entry);
        self.history.truncate(MAX_SAVED);
    }

    /// Load a saved entry back into the app.
    pub fn restore(&mut self, entry: &Entry) {
        self.mode = entry.mode;
        self.category = entry.category;
        self.from = entry
            .from
            .min(entry.category.units().len().saturating_sub(1));
        self.to = entry.to.min(entry.category.units().len().saturating_sub(1));
        self.input = entry.input.clone();
    }

    pub fn is_favorite(&self) -> bool {
        let entry = self.entry();
        self.favorites.contains(&entry)
    }

    /// Star / unstar the current conversion.
    pub fn toggle_favorite(&mut self) {
        let entry = self.entry();
        if let Some(pos) = self.favorites.iter().position(|e| e == &entry) {
            self.favorites.remove(pos);
        } else {
            self.favorites.insert(0, entry);
            self.favorites.truncate(MAX_SAVED);
        }
    }

    pub fn toggle_dark(&mut self) {
        self.dark = !self.dark;
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn clear_favorites(&mut self) {
        self.favorites.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_converts_one_meter_to_kilometer() {
        let app = App::new();
        assert_eq!(app.result_text(), "0.001 Kilometer");
    }

    #[test]
    fn calculator_mode_does_math() {
        let mut app = App::new();
        app.set_mode(Mode::Calculator);
        app.input = "1 + 1".to_string();
        assert_eq!(app.result_text(), "2");
        app.input = "2*(3+4)".to_string();
        assert_eq!(app.result_text(), "14");
    }

    #[test]
    fn invalid_input_is_reported() {
        let mut app = App::new();
        app.input = "1+".to_string();
        assert_eq!(app.result_text(), "(invalid expression)");
        assert!(!app.is_valid());
        app.input = String::new();
        assert_eq!(app.result_text(), "");
    }

    #[test]
    fn changing_category_resets_units() {
        let mut app = App::new();
        app.from = 5;
        app.to = 9;
        app.set_category(Category::Temperature);
        assert_eq!(app.from, 0);
        assert_eq!(app.to, 1);
    }

    #[test]
    fn swap_exchanges_units() {
        let mut app = App::new();
        app.set_to(3);
        app.swap();
        assert_eq!(app.from, 3);
        assert_eq!(app.to, 0);
    }

    #[test]
    fn search_filters_categories() {
        let mut app = App::new();
        app.search = "temp".to_string();
        assert_eq!(app.filtered_categories(), vec![Category::Temperature]);
        app.search = "zzz".to_string();
        assert!(app.filtered_categories().is_empty());
        app.search = String::new();
        assert_eq!(app.filtered_categories().len(), ALL_CATEGORIES.len());
    }

    #[test]
    fn history_saves_and_restores() {
        let mut app = App::new();
        app.save();
        assert_eq!(app.history().len(), 1);

        app.set_category(Category::Temperature);
        app.input = "100".to_string();
        app.save();
        assert_eq!(app.history().len(), 2);

        let first = app.history()[1].clone();
        app.restore(&first);
        assert_eq!(app.category, Category::Length);
        assert_eq!(app.input, "1");
    }

    #[test]
    fn history_deduplicates() {
        let mut app = App::new();
        app.save();
        app.save();
        assert_eq!(app.history().len(), 1);
    }

    #[test]
    fn invalid_input_is_not_saved() {
        let mut app = App::new();
        app.input = "1+".to_string();
        app.save();
        assert!(app.history().is_empty());
    }

    #[test]
    fn favorites_toggle() {
        let mut app = App::new();
        assert!(!app.is_favorite());
        app.toggle_favorite();
        assert!(app.is_favorite());
        assert_eq!(app.favorites().len(), 1);
        app.toggle_favorite();
        assert!(!app.is_favorite());
        assert!(app.favorites().is_empty());
    }

    #[test]
    fn format_is_applied() {
        let mut app = App::new();
        app.format = NumFormat::Dec3;
        assert_eq!(app.result_text(), "0.001 Kilometer");
        app.format = NumFormat::Dec0;
        assert_eq!(app.result_text(), "0 Kilometer");
    }

    #[test]
    fn entry_describe() {
        let app = App::new();
        assert_eq!(app.entry().describe(), "1 Meter -> Kilometer");
        let mut calc = App::new();
        calc.set_mode(Mode::Calculator);
        assert_eq!(calc.entry().describe(), "1 + 1");
    }
}
