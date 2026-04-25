# Monokakido Hover: Implementation Guide & Lessons Learned

This document outlines the high-performance text selection and zoom architecture implemented for mobile GPUI.

## 1. Hybrid Rendering Model (Performance)
**Problem:** Rendering long text character-by-character using thousands of `div` and `canvas` elements causes severe frame drops (down to 10fps) during scrolling.
**Solution:** Use two distinct rendering modes:
- **Idle Mode (Default):** Render the entire text as a single `div`. This is high-performance and ensures smooth 60fps scrolling.
- **Interaction Mode:** Switch to per-character `div` rendering *only* when a selection is active (e.g., `MouseButton::Right` is pressed).
- **Cleanup:** Immediately clear character bounds and metadata on `MouseUp` to release memory and return to Idle Mode.

## 2. The "Pending Resolve" Pattern (UX Accuracy)
**Problem:** When switching from Idle to Interaction mode, the component doesn't have character bounds yet. An immediate hit-test usually "hooks" to the first character (index 0), causing a jumpy anchor.
**Solution:**
1. On `MouseDown`, set `selection_range` to a dummy value to trigger Interaction Mode.
2. Store the touch position in `pending_anchor_position`.
3. In the next frame's `on_mouse_move`, use the now-available `char_bounds` to resolve that position to a precise character index.
4. This ensures the selection anchor is exactly under the finger even during the mode transition.

## 3. Language-Aware Word Snapping
**Problem:** CJK languages require character-level precision for dictionary lookups, while Western languages feel better with word-level snapping.
**Solution:** 
- Implement an `is_cjk(char)` check that includes:
    - Hiragana/Katakana (`\u{3040}-\u{30FF}`)
    - Kanji (`\u{4E00}-\u{9FFF}`)
    - CJK Symbols/Punctuation (`\u{3000}-\u{303F}`) — *Crucial for avoiding snapping to brackets.*
- **Western Snapping:** Expand selection to `\w+` boundaries, but **stop immediately** if a CJK character or bracket is encountered.

## 4. Zoom Header & Sliding Window
**Problem:** Highlighting large paragraphs makes the zoom header "wonky" (wrapping, expanding vertically, or losing the interaction point).
**Solution:**
- **Fixed Height:** Enforce a strict height on the Zoom bar and its content container.
- **Baseline Alignment:** Use `items_baseline()` for the text row to ensure different scripts (English/Japanese) line up perfectly.
- **Sliding Window:** Do not render the whole selection in the header. Instead, render a window centered on the finger (e.g., 7 chars before, 7 chars after). This keeps the interaction point visible and the UI stable.

## 5. Layout Integrity
- **Newline Handling:** In Interaction Mode (per-character flex), render a `div().w_full().h_0()` when encountering a `\n` to force a line break without adding height.
- **Wrapping:** Ensure the root container has `flex_wrap()` and the Idle Mode `div` has `w_full()` to match the layout of the character-grid.
- **Z-Index:** On mobile, overlays like the Zoom Header should be the **last child** of the root `Router` render function to ensure they appear above all other UI elements (TopAppBar, Search fields, etc.).
