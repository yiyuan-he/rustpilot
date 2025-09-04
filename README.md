# RustPilot

AI coding agent, but in Rust.

## Features

-  Read files from your file system
-  List directory contents
-  Edit files with diff preview
-  Interactive conversation interface
-  Extensible tool system

## Prerequisites

- Rust 1.70+
- An Anthropic API key

## Installation

1. Clone this repository
2. Copy `.env.example` to `.env` and add your Anthropic API key:
   ```bash
   cp .env.example .env
   # Edit .env and add your API key
   ```

3. Run:
   ```bash
   cargo run --release
   ```
