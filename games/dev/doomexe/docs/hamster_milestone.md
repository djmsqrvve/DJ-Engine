# Hamster Narrator – Milestone 1 Design

This document specifies the first vertical slice: an animated, procedural hamster narrator screen.

---

## Visual Target

- A large hamster portrait centered on screen, lit by candles.
- Strong pixel‑art style with visible CRT/retro artifacts.
- Expression and palette change with “corruption.”

Use the attached concept image as inspiration for proportions and vibe (not as strict pixel reference).

---

## Technical Requirements

### Resolution & Rendering

- Internal resolution: 320×240 or 400×240.
- Render all world elements to an offscreen texture at internal resolution.
- Upscale to window with nearest‑neighbor filtering.
- Apply post‑processing:
  - Scanlines.
  - Vignette / slight barrel distortion.
  - Optional chromatic aberration based on corruption.

### Hamster Rig

Parts (minimum):

- Body
- Head
- Left ear / right ear
- Left eye / right eye (multiple variants)
- Mouth (smile, neutral, angry, corrupted)
- Left paw / right paw (optional for Milestone 1)

Each part:

- Authored in Aseprite on a shared canvas.
- Exported as a trimmed sprite + JSON metadata containing original offset.
- At runtime, `HamsterPart` stores:
  - Part type (enum).
  - Texture handle.
  - Offset from origin (in pixels).
  - Draw layer/z‑index.

### Components

- `HamsterNarrator`
  - `corruption: f32`
  - `expression: u8` or enum
  - `animation_time: f32`
  - `mood` (optional high‑level enum)
- `HamsterPart`
  - `part_type: PartType`
  - `offset: Vec2`
  - `layer: u32`
  - `sprite_index` or direct handle

### Systems

1. **Assembly**
   - On startup, spawn one hamster entity with children for each part.
   - Apply transforms so assembled hamster matches Aseprite layout.

2. **Breathing**
   - Sine wave based scale:
     - `scale_y = 1.0 + A * sin(ωt)`
     - `scale_x = 1.0 / scale_y` (approximate area constancy)
   - Applied at the body/root; head/ears follow automatically.

3. **Blinking**
   - Timer that toggles eye variant / alpha.
   - Randomized interval in a range (e.g., 3–7 seconds) to avoid robotic timing.

4. **Idle Motion**
   - Slight head rotation or position offset based on noise or slow sine.

5. **Corruption Effects**
   - Corruption ∈ [0, 100].
   - Drives:
     - Palette shift index (shader uniform).
     - CRT intensity.
     - Optional raster jitter amplitude.

6. **Input / Debug Controls**
   - Keys to:
     - Increase/decrease corruption.
     - Cycle expressions.
     - Trigger “dialogue” events routed through Lua.

---

## Lua Integration (Minimal)

Lua script responsibilities in Milestone 1:

- Map simple inputs (e.g., A/S/D) to state changes:
  - “Nice” choice: decrease corruption, set happy expression.
  - “Mean” choice: increase corruption, set angry/corrupted expression.
- Later, expand to full dialogue trees.

Rust → Lua API (first pass):

- `set_corruption(f32)`
- `set_expression(u8 or string)`
- `log(message)` for debugging

---

## Success Criteria

- Hamster appears on screen and is clearly composed from parts.
- Breathing loop feels organic, not mechanical.
- Blinking and subtle idle motion are visible.
- Corruption slider or keys clearly change:
  - Palette / color mood.
  - CRT/post‑processing intensity.
- Lua script can trigger a noticeable change in expression/corruption without restarting the game.
- Prototype runs at 60fps on a modest desktop/laptop.
