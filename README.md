# diffkit

[![Build Status](https://github.com/sphaso/diffkit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/sphaso/diffkit/actions/workflows/ci.yml)

A Rust library for diffing and patching sequences and nested structures.

## Features

- **Myers diff** — efficient sequence diffing via the Myers algorithm
- **Recursive diff** — structural diffing of nested maps and sequences
- **Hunks** — group changes with context lines
- **Unified diff** — serialize and deserialize patches in unified diff format

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
diffkit = "0.1.0"
```

## Usage

### Sequence diff

```rust
use diffkit::myers::diff;
use diffkit::patch::{apply, hunks};
use diffkit::serialization::ToPatch;

let old = vec!["hello", "world"];
let new = vec!["hello", "rust"];
let myers_edits = diff(&old, &new);
```

### Recursive diff

```rust
use std::collections::HashMap;
use diffkit::recursive::{apply, diff};
use diffkit::patch::hunks;

let mut old = HashMap::new();
old.insert("Hello".to_string(), 1);
let mut new = HashMap::new();
new.insert("Hello".to_string(), 2);
let changes = diff(&old, &new);

let equal_to_new = apply(&old, &changes);
```

### Applying a patch

```rust
use diffkit::myers::diff;
use diffkit::patch::{apply, hunks};
use diffkit::serialization::ToPatch;

let old = vec!["hello", "world"];
let new = vec!["hello", "rust"];
let myers_edits = diff(&old, &new);

let hunks = hunks(myers_edits);
let patch = hunks.to_patch(Some("lib.rs"), Some("lib.rs"));

let equal_to_new = apply(&old, &hunks);
```

## License
See [UNLICENSE](UNLICENSE) for details.
