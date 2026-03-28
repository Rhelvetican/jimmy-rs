# JimmyRS

A type‑safe JSON builder for Rust that leverages the typestate pattern to guarantee syntactically correct JSON at compile time.

## Overview

This crate provides a fluent API for constructing JSON data incrementally.

The builder’s state machine tracks the current position in the JSON structure – root, object, array, or field – and only allows valid transitions.

This means you can’t accidentally write a value where a field name is expected, or close an object before it was opened, etc. All these errors are caught by the compiler.

## Installation

Add this to your `Cargo.toml` dependencies section:

```toml
jimmy-rs = "1.0.1"
```

## Disclaimer

Most of the code are originally written by `xeondev`.
