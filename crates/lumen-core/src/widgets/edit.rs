//! Single-line text editing model: cursor, selection, grapheme- and
//! word-aware movement, undo. Pure and independent of rendering.

use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextEdit {
    /// Byte index of the caret.
    pub cursor: usize,
    /// Byte index of the selection anchor (== cursor when nothing selected).
    pub anchor: usize,
    undo: Vec<(String, usize, usize)>,
    redo: Vec<(String, usize, usize)>,
    /// Coalesce consecutive typed characters into one undo step.
    typing_run: bool,
    /// Input-method composition inside the text (byte range), shown
    /// underlined until it's committed.
    pub marked: Option<std::ops::Range<usize>>,
}

fn floor_boundary(s: &str, i: usize) -> usize {
    let mut i = i.min(s.len());
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn prev_grapheme(s: &str, i: usize) -> usize {
    s[..i].grapheme_indices(true).next_back().map(|(j, _)| j).unwrap_or(0)
}

fn next_grapheme(s: &str, i: usize) -> usize {
    s[i..].graphemes(true).next().map(|g| i + g.len()).unwrap_or(s.len())
}

fn is_word(g: &str) -> bool {
    g.chars().any(|c| c.is_alphanumeric() || c == '_')
}

fn prev_word(s: &str, i: usize) -> usize {
    let mut pos = i;
    let mut seen_word = false;
    for (j, w) in s[..i].split_word_bound_indices().rev() {
        if is_word(w) {
            seen_word = true;
            pos = j;
        } else if seen_word {
            break;
        } else {
            pos = j;
        }
    }
    pos
}

fn next_word(s: &str, i: usize) -> usize {
    let mut pos = i;
    let mut seen_word = false;
    for (j, w) in s[i..].split_word_bound_indices() {
        let end = i + j + w.len();
        if is_word(w) {
            seen_word = true;
            pos = end;
        } else if seen_word {
            break;
        } else {
            pos = end;
        }
    }
    pos
}

impl TextEdit {
    pub fn has_selection(&self) -> bool {
        self.cursor != self.anchor
    }

    pub fn selection(&self) -> std::ops::Range<usize> {
        self.cursor.min(self.anchor)..self.cursor.max(self.anchor)
    }

    pub fn selected<'a>(&self, s: &'a str) -> &'a str {
        &s[self.selection()]
    }

    /// Clamp indices after the text changed from outside.
    pub fn clamp(&mut self, s: &str) {
        self.cursor = floor_boundary(s, self.cursor);
        self.anchor = floor_boundary(s, self.anchor);
        if self.marked.as_ref().is_some_and(|m| m.end > s.len() || !s.is_char_boundary(m.start) || !s.is_char_boundary(m.end)) {
            self.marked = None;
        }
    }

    /// The range an input-method event replaces: the composition, else
    /// `replace` (clamped to the text), else the selection.
    fn ime_range(&mut self, s: &str, replace: &Option<std::ops::Range<usize>>) -> std::ops::Range<usize> {
        if let Some(m) = self.marked.take() {
            m
        } else if let Some(r) = replace {
            let a = floor_boundary(s, r.start);
            a..floor_boundary(s, r.end).max(a)
        } else {
            self.selection()
        }
    }

    /// Apply an input-method event. The composition lives in the text,
    /// marked, until it's committed; one undo step covers the whole
    /// composition. Returns true if the text changed.
    pub fn ime(&mut self, s: &mut String, ev: &crate::input::Ime) -> bool {
        use crate::input::Ime;
        let before = s.len();
        let composing = self.marked.is_some();
        match ev {
            Ime::Preedit { text, cursor, replace } => {
                if !composing && text.is_empty() {
                    return false;
                }
                if !composing {
                    self.snapshot(s, false);
                }
                let text: String = text.chars().filter(|c| !c.is_control()).collect();
                let r = self.ime_range(s, replace);
                let old = s[r.clone()].to_owned();
                s.replace_range(r.clone(), &text);
                if text.is_empty() {
                    self.cursor = r.start;
                    self.anchor = r.start;
                } else {
                    self.marked = Some(r.start..r.start + text.len());
                    let (a, b) = cursor.unwrap_or((text.len(), text.len()));
                    self.anchor = r.start + floor_boundary(&text, a);
                    self.cursor = r.start + floor_boundary(&text, b);
                }
                self.typing_run = false;
                old != text || s.len() != before
            }
            Ime::Commit { text, replace } => {
                let text: String = text.chars().filter(|c| !c.is_control()).collect();
                if !composing {
                    if text.is_empty() && replace.is_none() {
                        return false;
                    }
                    self.snapshot(s, false);
                }
                let r = self.ime_range(s, replace);
                let old = s[r.clone()].to_owned();
                s.replace_range(r.clone(), &text);
                self.cursor = r.start + text.len();
                self.anchor = self.cursor;
                self.typing_run = false;
                old != text
            }
        }
    }

    fn snapshot(&mut self, s: &str, coalesce: bool) {
        if coalesce && self.typing_run {
            return;
        }
        self.undo.push((s.to_owned(), self.cursor, self.anchor));
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.typing_run = coalesce;
    }

    fn delete_selection(&mut self, s: &mut String) {
        self.marked = None;
        let r = self.selection();
        s.replace_range(r.clone(), "");
        self.cursor = r.start;
        self.anchor = r.start;
    }

    /// Insert text at the caret, replacing any selection.
    pub fn insert(&mut self, s: &mut String, text: &str) {
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        if text.is_empty() {
            return;
        }
        let coalesce = !self.has_selection() && !text.contains(' ') && text.chars().count() == 1;
        self.snapshot(s, coalesce);
        self.delete_selection(s);
        s.insert_str(self.cursor, &text);
        self.cursor += text.len();
        self.anchor = self.cursor;
    }

    /// Insert a line break at the caret (multi-line editors).
    pub fn insert_newline(&mut self, s: &mut String) {
        self.snapshot(s, false);
        self.delete_selection(s);
        s.insert(self.cursor, '\n');
        self.cursor += 1;
        self.anchor = self.cursor;
    }

    pub fn backspace(&mut self, s: &mut String, word: bool, line: bool) {
        self.marked = None;
        if self.has_selection() {
            self.snapshot(s, false);
            self.delete_selection(s);
            return;
        }
        if self.cursor == 0 {
            return;
        }
        self.snapshot(s, false);
        let from = if line {
            0
        } else if word {
            prev_word(s, self.cursor)
        } else {
            prev_grapheme(s, self.cursor)
        };
        s.replace_range(from..self.cursor, "");
        self.cursor = from;
        self.anchor = from;
    }

    pub fn delete(&mut self, s: &mut String, word: bool) {
        self.marked = None;
        if self.has_selection() {
            self.snapshot(s, false);
            self.delete_selection(s);
            return;
        }
        if self.cursor >= s.len() {
            return;
        }
        self.snapshot(s, false);
        let to = if word { next_word(s, self.cursor) } else { next_grapheme(s, self.cursor) };
        s.replace_range(self.cursor..to, "");
    }

    fn place(&mut self, i: usize, extend: bool) {
        self.marked = None;
        self.cursor = i;
        if !extend {
            self.anchor = i;
        }
        self.typing_run = false;
    }

    pub fn left(&mut self, s: &str, extend: bool, word: bool) {
        if self.has_selection() && !extend && !word {
            let r = self.selection();
            return self.place(r.start, false);
        }
        let i = if word { prev_word(s, self.cursor) } else { prev_grapheme(s, self.cursor) };
        self.place(i, extend);
    }

    pub fn right(&mut self, s: &str, extend: bool, word: bool) {
        if self.has_selection() && !extend && !word {
            let r = self.selection();
            return self.place(r.end, false);
        }
        let i = if word { next_word(s, self.cursor) } else { next_grapheme(s, self.cursor) };
        self.place(i, extend);
    }

    pub fn home(&mut self, extend: bool) {
        self.place(0, extend);
    }

    pub fn end(&mut self, s: &str, extend: bool) {
        self.place(s.len(), extend);
    }

    pub fn select_all(&mut self, s: &str) {
        self.marked = None;
        self.anchor = 0;
        self.cursor = s.len();
    }

    /// Select the word around byte `i` (double click).
    pub fn select_word(&mut self, s: &str, i: usize) {
        self.marked = None;
        for (j, w) in s.split_word_bound_indices() {
            if i >= j && i <= j + w.len() && (is_word(w) || i < j + w.len()) {
                self.anchor = j;
                self.cursor = j + w.len();
                return;
            }
        }
        self.place(i, false);
    }

    /// Place the caret from a pointer (extend with shift).
    pub fn click(&mut self, i: usize, extend: bool) {
        self.place(i, extend);
    }

    pub fn undo(&mut self, s: &mut String) {
        self.marked = None;
        if let Some((t, c, a)) = self.undo.pop() {
            self.redo.push((std::mem::replace(s, t), self.cursor, self.anchor));
            self.cursor = c;
            self.anchor = a;
            self.typing_run = false;
        }
    }

    pub fn redo(&mut self, s: &mut String) {
        self.marked = None;
        if let Some((t, c, a)) = self.redo.pop() {
            self.undo.push((std::mem::replace(s, t), self.cursor, self.anchor));
            self.cursor = c;
            self.anchor = a;
            self.typing_run = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ed(s: &str, cursor: usize) -> (TextEdit, String) {
        (TextEdit { cursor, anchor: cursor, ..Default::default() }, s.to_owned())
    }

    #[test]
    fn insert_and_replace_selection() {
        let (mut e, mut s) = ed("hello", 5);
        e.insert(&mut s, " world");
        assert_eq!(s, "hello world");
        e.select_all(&s);
        e.insert(&mut s, "x");
        assert_eq!(s, "x");
        assert_eq!(e.cursor, 1);
    }

    #[test]
    fn control_chars_filtered() {
        let (mut e, mut s) = ed("", 0);
        e.insert(&mut s, "a\nb\tc");
        assert_eq!(s, "abc");
    }

    #[test]
    fn grapheme_aware_backspace_and_moves() {
        let (mut e, mut s) = ed("e\u{301}x", 4); // é as e + combining acute, then x
        e.left(&s, false, false);
        assert_eq!(e.cursor, 3);
        e.backspace(&mut s, false, false);
        assert_eq!(s, "x");
        let (mut e, mut s) = ed("a👍🏽b", 9);
        e.backspace(&mut s, false, false);
        assert_eq!(s, "ab");
    }

    #[test]
    fn word_jumps() {
        let s = "deploy to production now";
        let mut e = TextEdit::default();
        e.right(s, false, true);
        assert_eq!(e.cursor, 6);
        e.right(s, false, true);
        assert_eq!(e.cursor, 9);
        e.end(s, false);
        e.left(s, false, true);
        assert_eq!(e.cursor, 21);
        e.left(s, false, true);
        assert_eq!(e.cursor, 10);
    }

    #[test]
    fn word_backspace() {
        let (mut e, mut s) = ed("foo bar baz", 11);
        e.backspace(&mut s, true, false);
        assert_eq!(s, "foo bar ");
        e.backspace(&mut s, true, false);
        assert_eq!(s, "foo ");
    }

    #[test]
    fn shift_extends_and_collapse() {
        let s = "abcdef";
        let mut e = TextEdit { cursor: 2, anchor: 2, ..Default::default() };
        e.right(s, true, false);
        e.right(s, true, false);
        assert_eq!(e.selection(), 2..4);
        assert_eq!(e.selected(s), "cd");
        e.left(s, false, false);
        assert_eq!((e.cursor, e.anchor), (2, 2));
    }

    #[test]
    fn select_word_on_double_click() {
        let s = "rk_live_9d2f key";
        let mut e = TextEdit::default();
        e.select_word(s, 3);
        assert_eq!(e.selected(s), "rk_live_9d2f");
    }

    #[test]
    fn undo_redo_coalesces_typing() {
        let (mut e, mut s) = ed("", 0);
        for c in ["a", "b", "c"] {
            e.insert(&mut s, c);
        }
        e.insert(&mut s, " ");
        e.insert(&mut s, "d");
        assert_eq!(s, "abc d");
        e.undo(&mut s);
        assert_eq!(s, "abc ");
        e.undo(&mut s);
        assert_eq!(s, "abc");
        e.undo(&mut s);
        assert_eq!(s, "");
        e.redo(&mut s);
        assert_eq!(s, "abc");
    }

    #[test]
    fn clamp_after_external_change() {
        let mut e = TextEdit { cursor: 10, anchor: 8, ..Default::default() };
        e.clamp("héllo");
        assert_eq!(e.cursor, 6);
        let mut e = TextEdit { cursor: 2, anchor: 2, ..Default::default() };
        e.clamp("héllo");
        assert_eq!(e.cursor, 1);
    }

    fn pre(t: &str) -> crate::input::Ime {
        crate::input::Ime::Preedit { text: t.into(), cursor: None, replace: None }
    }
    fn commit(t: &str) -> crate::input::Ime {
        crate::input::Ime::Commit { text: t.into(), replace: None }
    }

    #[test]
    fn ime_composition_is_marked_then_committed() {
        let (mut e, mut s) = ed("ab", 1);
        e.ime(&mut s, &pre("k"));
        assert_eq!((s.as_str(), e.marked.clone(), e.cursor), ("akb", Some(1..2), 2));
        e.ime(&mut s, &pre("か"));
        assert_eq!((s.as_str(), e.marked.clone(), e.cursor), ("aかb", Some(1..4), 4));
        e.ime(&mut s, &commit("可"));
        assert_eq!((s.as_str(), e.marked.clone(), e.cursor), ("a可b", None, 4));
        // One undo step for the whole composition.
        e.undo(&mut s);
        assert_eq!(s, "ab");
    }

    #[test]
    fn ime_preedit_replaces_selection_and_cancels_cleanly() {
        let mut e = TextEdit { cursor: 3, anchor: 1, ..Default::default() };
        let mut s = String::from("abcd");
        e.ime(&mut s, &crate::input::Ime::Preedit { text: "にほ".into(), cursor: Some((0, 3)), replace: None });
        assert_eq!(s, "aにほd");
        assert_eq!((e.anchor, e.cursor), (1, 4));
        e.ime(&mut s, &pre(""));
        assert_eq!((s.as_str(), e.marked.clone(), e.cursor), ("ad", None, 1));
    }

    #[test]
    fn ime_commit_with_replacement_range() {
        // macOS press-and-hold: "e" was typed, then "é" replaces it.
        let (mut e, mut s) = ed("cafe", 4);
        e.ime(&mut s, &crate::input::Ime::Commit { text: "é".into(), replace: Some(3..4) });
        assert_eq!((s.as_str(), e.cursor), ("café", 5));
    }

    #[test]
    fn moving_the_caret_ends_the_composition() {
        let (mut e, mut s) = ed("", 0);
        e.ime(&mut s, &pre("ni"));
        e.left(&s, false, false);
        assert_eq!((s.as_str(), e.marked.clone()), ("ni", None));
    }
}
