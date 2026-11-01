# Pafta

A tool for designing egui interfaces by hand. Drag widgets into a window and lay them out;
Pafta turns the design into a single-file eframe app that runs with `cargo run`.

**Download:** [Pafta.exe](https://github.com/DennisYildizdev/pafta/raw/main/dist/Pafta.exe)
(Windows 10/11, no install needed)

```
cargo run --release
```

## Single-file exe

```
cargo build --release
```

`target/release/pafta.exe` is portable on its own; a copy lives at `dist/Pafta.exe` so it is
easy to find. The icon and version info are embedded in the exe (`build.rs`), and so is the
Visual C++ runtime (`.cargo/config.toml`). It opens on any other Windows 10/11 machine
without installing anything.

## Language

The interface opens in English. For Turkish: View → Language → Türkçe. The choice is saved.
The language sets the menus, names of new widgets (`button_1` / `dugme_1`), tool shortcut
letters and comments in the generated code. It never touches the text in your design.

## How it works

- **Components** (left): drag a widget onto the canvas, or click it and draw on the canvas.
  The shortcut letter is shown on the right of each row (T label, B button…; in Turkish
  E etiket, D düğme…).
- **Canvas** (middle): the window is drawn as a flat sheet. While moving, widgets snap to
  neighbouring edges and centres (pink lines), and distances show as measure lines. Hold Alt
  to turn snapping off and show measures. Drag the window's bottom-right corner to resize it.
- **Properties** (right): name, text, items, position and size; align and distribute when
  several widgets are selected.
- **Code** (bottom): `main.rs` is rewritten as the design changes. Save it with Ctrl+E, or use
  File → Export Cargo Project to write a working project into an empty folder.
- **Preview** (F5): runs the design in a separate window with the same egui widgets as the
  generated code.

Designs are JSON files with the `.pafta` extension. Pafta remembers where you left off when it closes.

## Shortcuts

| Action | Key |
| --- | --- |
| Widget tool (English) | T B F A C R D L S P G |
| Widget tool (Turkish) | E D G M O S A L K I C |
| Edit text in place | Enter or double-click |
| Move | Arrows (Shift: grid step) |
| Duplicate, delete | Ctrl+D, Delete |
| Undo, redo | Ctrl+Z, Ctrl+Y |
| Bring forward, send backward | PgUp, PgDn (Shift: to front / back) |
| Zoom | Ctrl + wheel, Ctrl+0 actual size |
| Pan | Wheel, Space + drag |
| Code panel, preview | Ctrl+J, F5 |

Press F1 in the app to see them all.

## Development

```
cargo test
```

UI tests drive the canvas with a virtual mouse (`src/ui_tests.rs`). To save the frames as
images: `PAFTA_GORUNTU=folder cargo test`. To print the example design's code:
`cargo run --example ornek_kod` (add `-- tr` for Turkish output).

`src/live.rs` both runs the preview and holds the helpers the code generator copies into its
output, so what you see in the preview is the generated app itself.
