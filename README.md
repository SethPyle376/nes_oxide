# nes_oxide

A simple NES emulator built in Rust for fun and learning.

Currently capable of playing simple early NES games like Donkey Kong or Pacman.

## Usage

`cargo run --package nes_oxide --bin nes_oxide -- --rom <PATH_TO_NES_ROM>`

## Features
- ✅ = Done
- 🚧 = In Progress
- 📋 = Planned
- ❌ = Not Planned / Out of scope


- CPU
  - ✅Official Opcodes
  - 📋Unofficial Opcodes
  - ❌Cycle Accurate
- PPU
  - ✅Static Background Rendering
  - ✅Sprite Rendering
  - 🚧Scrolling Background Rendering
- Mappers
  - ✅Mapper 0
  - 📋Other Mappers
- Joypads
  - ✅Joypad 1
  - 🚧Joypad 2
  - 📋SDL Gamepad Support
- Debug
  - 🚧VRAM Viewer Widget
  - 🚧PPU Status Viewer Widget
- 📋WASM Build / Online Version
- 📋APU

## Media

![pacman](/media/pac.png)
![dk](/media/dk.png)