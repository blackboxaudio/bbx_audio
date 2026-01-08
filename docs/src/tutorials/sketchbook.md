# Sketch Discovery with Sketchbook

This tutorial covers the sketch discovery and management system in bbx_draw.

## Prerequisites

Enable the sketchbook feature (enabled by default):

```toml
[dependencies]
bbx_draw = { version = "0.1", features = ["sketchbook"] }
```

## The Sketch Trait

Implement `Sketch` to make your sketch discoverable:

```rust
use bbx_draw::sketch::Sketch;
use nannou::{App, Frame, event::Update};

pub struct MySketch {
    time: f32,
}

impl Sketch for MySketch {
    fn name(&self) -> &str {
        "My Sketch"
    }

    fn description(&self) -> &str {
        "A simple animated sketch"
    }

    fn model(app: &App) -> Self {
        app.new_window().build().unwrap();
        Self { time: 0.0 }
    }

    fn update(&mut self, _app: &App, update: Update) {
        self.time += update.since_last.as_secs_f32();
    }

    fn view(&self, app: &App, frame: Frame) {
        let draw = app.draw();
        draw.background().color(nannou::color::BLACK);
        // Draw something...
        draw.to_frame(app, &frame).unwrap();
    }
}
```

## Creating a Sketchbook

```rust
use bbx_draw::sketch::Sketchbook;

fn main() -> std::io::Result<()> {
    // Uses platform cache directory
    let sketchbook = Sketchbook::new()?;
    Ok(())
}
```

Default cache locations:
- Linux: `~/.cache/bbx_draw/sketches`
- macOS: `~/Library/Caches/bbx_draw/sketches`
- Windows: `%LOCALAPPDATA%\bbx_draw\sketches`

### Custom Cache Directory

```rust
use bbx_draw::sketch::Sketchbook;
use std::path::PathBuf;

let cache_dir = PathBuf::from("./my_sketches");
let sketchbook = Sketchbook::with_cache_dir(cache_dir)?;
```

## Discovering Sketches

Scan a directory for sketch files:

```rust
use bbx_draw::sketch::Sketchbook;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let mut sketchbook = Sketchbook::new()?;

    let sketches_dir = PathBuf::from("./sketches");
    let count = sketchbook.discover(&sketches_dir)?;

    println!("Discovered {} sketches", count);
    Ok(())
}
```

The `discover` method:
1. Scans the directory for `.rs` files
2. Extracts metadata from doc comments
3. Caches results for future use

## Managing Sketches

### Listing All Sketches

```rust
for sketch in sketchbook.list() {
    println!("{}: {}", sketch.name, sketch.description);
}
```

### Finding by Name

```rust
if let Some(sketch) = sketchbook.get("My Sketch") {
    println!("Found: {:?}", sketch.source_path);
}
```

### Manual Registration

```rust
use bbx_draw::sketch::SketchMetadata;
use std::time::SystemTime;
use std::path::PathBuf;

let metadata = SketchMetadata {
    name: "Custom Sketch".to_string(),
    description: "Manually registered".to_string(),
    source_path: PathBuf::from("./custom.rs"),
    last_modified: SystemTime::now(),
};

sketchbook.register(metadata)?;
```

### Removing Sketches

```rust
if let Some(removed) = sketchbook.remove("Old Sketch") {
    println!("Removed: {}", removed.name);
}
```

## Metadata Extraction

The sketchbook extracts descriptions from doc comments (`//!`):

```rust
//! A waveform visualizer sketch.
//! Shows audio waveforms in real-time.

use bbx_draw::*;
// ...
```

This becomes:
```
name: "waveform_visualizer" (from filename)
description: "A waveform visualizer sketch. Shows audio waveforms in real-time."
```

## SketchMetadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Display name (from filename) |
| `description` | `String` | From doc comments |
| `source_path` | `PathBuf` | Path to source file |
| `last_modified` | `SystemTime` | File modification time |

## Example: Sketch Browser

```rust
use bbx_draw::sketch::Sketchbook;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let mut sketchbook = Sketchbook::new()?;

    // Discover sketches in current directory
    let dir = PathBuf::from("./sketches");
    sketchbook.discover(&dir)?;

    // List available sketches
    println!("Available sketches:");
    for (i, sketch) in sketchbook.list().iter().enumerate() {
        println!("  {}. {} - {}", i + 1, sketch.name, sketch.description);
    }

    // Get user selection
    let name = "waveform_basic";
    if let Some(sketch) = sketchbook.get(name) {
        println!("\nRunning: {}", sketch.name);
        println!("Source: {:?}", sketch.source_path);
        // Launch the sketch...
    }

    Ok(())
}
```

## Caching

The sketchbook persists metadata to `registry.json` in the cache directory:

```json
[
  {
    "name": "waveform_basic",
    "description": "Basic waveform visualizer",
    "source_path": "/path/to/waveform_basic.rs",
    "last_modified": 1704067200
  }
]
```

Cache is automatically:
- Loaded on `new()` or `with_cache_dir()`
- Updated on `discover()`, `register()`, `remove()`

## Next Steps

- [Visualizer Trait](../crates/draw/visualizer.md) - Build visualizers for your sketches
- [Real-Time Visualization](visualization.md) - Audio visualization tutorial
