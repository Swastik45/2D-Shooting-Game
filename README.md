# My Bevy Game

A 2D top-down action game built with the [Bevy](https://bevyengine.org/) game engine in Rust.

## Overview

This is a dynamic action game where you control a player character that must survive waves of enemies. Defeat enemies by shooting them with your gun while managing your health and avoiding enemy fire.

![Gameplay Screenshot](./Screenshot%20from%202026-04-30%2018-26-25.png)

## Features

- **Player Character**: Smooth movement with directional animation states (idle, walking front, walking side, walking back)
- **Combat System**: Fire bullets at enemies with realistic cooldown mechanics and muzzle flash effects
- **Enemy AI**: Enemies spawn dynamically and intelligently move toward the player while firing back
- **Collision Detection**: Bullet-to-enemy and enemy-bullet-to-player collision detection
- **Health & Score System**: Track player health and cumulative score
- **Game States**: Playing and Game Over states with restart functionality
- **Tile-based World**: Walkable and non-walkable terrain with map boundaries
- **Camera System**: Dynamic camera that follows the player

## Controls

- **Movement**: Use arrow keys or WASD to move
- **Aim**: Mouse cursor follows your aim
- **Shoot**: Left mouse button to fire at enemies
- **Restart**: Press R when the game is over

## Building & Running

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Linux/MacOS/Windows with development dependencies for Bevy

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release
```

### Development

For faster compilation during development:

```bash
cargo run
```

## Project Structure

```
src/
├── main.rs           # Application entry point and ECS setup
├── player.rs         # Player character, movement, and animation
├── weapon.rs         # Gun mechanics, bullet firing, and visuals
├── enemy.rs          # Enemy spawning, movement, and AI
├── combat.rs         # Collision detection and damage system
├── camera.rs         # Camera setup and following behavior
├── world.rs          # Tile-based world and map generation
├── game_state.rs     # Game state management and logic
├── game_ui.rs        # HUD, health display, and game over screen
```

## Game Mechanics

### Player
- Controlled via keyboard input
- Animated based on movement direction
- Equipped with a gun that fires bullets
- Has health that decreases when hit by enemy bullets
- Game ends when health reaches zero

### Enemies
- Spawn periodically at set intervals
- Move toward the player's position
- Fire bullets back at the player
- Defeated when hit by player bullets
- Contribute to the player's score when defeated

### Combat
- Bullets travel in a straight line until hitting enemies or leaving the map
- Muzzle flash effects provide visual feedback
- Collisions are checked each frame for accuracy
- Health system for both player and enemies

## Dependencies

- **bevy**: 0.18.1 - The game engine
- **directories**: 5.0 - Cross-platform directory utilities

## Configuration

Game parameters can be adjusted in the source files:
- `PLAYER_SPEED` - Player movement speed (in player.rs)
- `ANIMATION_SPEED` - Player animation frame rate (in player.rs)
- `FIRE_COOLDOWN` - Time between shots (in player.rs)
- Tile size and map dimensions (in world.rs)

## Future Enhancements

- Multiple weapon types
- Power-up items
- Different enemy types with unique behaviors
- Sound effects and background music
- Difficulty levels
- Leaderboard/high score saving
- Menu system

## License

This project is open source and available for personal use and modification.

## Author

Created with ❤️ using Bevy Engine
