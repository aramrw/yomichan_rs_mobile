# Design Spec: Custom Long-Press & Selection Handles (The Monokakido Way)

## Goal
Implement a manual, high-performance text selection system for iOS that bypasses native `UITextInput` selection handles. This provides 100% control over the "feel", snappiness, and the "Action Bar" (Lookup/Copy/etc.) to match the ergonomic quality of high-end dictionary apps like Monokakido.

## 1. Custom Long-Press Gesture (GPUI/Rust)
Currently, `IosWindow` only distinguishes between a **Tap** and a **Scroll**. We will extend this to include a **Long-Press**.

### Detection Strategy
*   **Location:** `src/ios/window.rs` inside `handle_touch`.
*   **Mechanism:** 
    *   Add a `start_time: Instant` to the `TouchState::Pending` variant.
    *   On `UITouchPhase::Began`, record the current time.
    *   On `UITouchPhase::Stationary` or `Moved` (within `SCROLL_SLOP`), check if `Instant::now() - start_time > 400ms`.
    *   If the duration is exceeded and the touch hasn't moved beyond the slop, emit a new `PlatformInput::LongPress` event (to be added to GPUI core or handled via a custom `PlatformInput` variant).
*   **Fine-tuning:** Allow a small "wobble" (the existing `SCROLL_SLOP`) during the hold duration.

## 2. Hit-Testing & Word Selection
Once a `LongPress` is detected at `(x, y)`:
*   **TextInput Component:** Convert the screen coordinates to local text offsets.
*   **Word Extraction:** Use the `TextField` data model to find the word boundaries around the cursor.
    *   Traverse backwards from the cursor until a separator (space, punctuation, or CJK boundary) is hit.
    *   Traverse forwards from the cursor until a separator is hit.
*   **Selection State:** Update `TextField.selection` with these boundaries.

## 3. Custom Selection UI (GPUI)
Since we are not using native iOS handles, we must render them ourselves in GPUI.

### Selection Handles
*   **Rendering:** Add two handle elements to the `TextInput` render tree.
*   **Positioning:** Absolutely position them at the start and end of the selection range.
*   **Interaction:** The handles should be draggable. Moving a handle updates the corresponding `selection` boundary in the `TextField`.

### Action Bar (Popup Menu)
*   **Component:** A custom GPUI `div` with absolute positioning.
*   **Logic:** Appears above the selection range.
*   **Buttons:**
    *   **Copy:** Copy selected text to clipboard.
    *   **Lookup:** (Custom) Trigger a dictionary lookup.
    *   **Translate:** (Custom) Trigger a translation.
*   **Feel:** Instant appearance without the native iOS "fade and float" delay.

## 4. Performance & Drawing Feel
*   **MouseMove Support:** Update the selection range in real-time as the user drags their finger between handles.
*   **No Native Magician:** Bypass the native iOS magnifier and selection loupe to ensure zero-latency feedback.

## 5. Reference: `gpui-component`
*   **Goal:** Once the new session is started, clone `gpui-component` from GitHub.
*   **Integration:** Analyze how they handle complex interactions (if any) and adapt their component patterns for the `TextInput` and `ActionBar` to ensure best practices.

## 6. Success Criteria
1.  Holding a finger on a word for 400ms highlights the entire word.
2.  Custom selection handles appear and are draggable.
3.  A custom Action Bar appears with "Copy" and "Lookup" options.
4.  Zero interference from native iOS selection handles.
