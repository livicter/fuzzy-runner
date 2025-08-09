# fuzzy runner: Cyberpunk Rooftop Run

Welcome to a fast-paced, endless platformer set against the backdrop of a sprawling, neon-drenched cyberpunk city. Built with the Bevy game engine in Rust, this game challenges you to survive for as long as you can, navigating treacherous rooftops while fending off a relentless zombie horde.
<img width="1591" height="925" alt="screen_cap1" src="https://github.com/user-attachments/assets/caee4440-7fa2-45f8-9a6c-8fe470f9bdd1" />

-----

## Setting

The game is set in a dystopian future city, characterized by its towering skyscrapers and vibrant, glowing advertisements. You'll be running and jumping across rooftops high above the city streets, with a beautiful parallax background that gives a sense of depth and scale to the urban environment.

-----

## Game Mechanics

### Endless Challenge

The primary goal is to travel as far as you can. The platforms are procedurally generated, meaning every run is a unique experience. Your distance is tracked in the top-left corner, serving as your score. Compete against yourself to beat your own highest record\!

### Health & Survival

You have a health bar, also displayed at the top-left. Your survival depends on two things:

1.  **Avoiding the Fall:** The city is a long way down. Falling off a platform into the abyss will end your run instantly.
2.  **Managing Enemies:** The rooftops are infested with zombies\!

### The Zombie Menace

Zombies are your primary obstacle. They are programmed with a simple but effective AI:
<img width="1588" height="931" alt="screen_cap2" src="https://github.com/user-attachments/assets/2e37c9f7-0dfc-4e5b-abf6-35d87f5a19c1" />

  * They will relentlessly chase you.
  * They can jump across gaps and onto platforms to keep up with you.
  * **If a zombie touches you, it will drain your health.**

When your health bar is fully depleted, or if you fall, the game is over.

-----

## How to Play

The controls are simple and responsive, designed for fluid movement.

  * **Move Left:** `A` or `←` (Left Arrow)
  * **Move Right:** `D` or `→` (Right Arrow)
  * **Jump:** `W`, `↑` (Up Arrow), or `Spacebar`
  * **Pause Game:** `Escape`

-----

## Features

### Pause & Settings Menus

You can press `Escape` at any time to bring up the pause menu, which gives you the option to:
<img width="1590" height="918" alt="screen_cap3" src="https://github.com/user-attachments/assets/3fd7fe37-50e9-4c79-bdb5-5fb61c467ea8" />

  * **Resume:** Jump straight back into the action.
  * **Reset:** End your current run and start a new one.
  * **Settings:** Open the settings menu.


### Customizable Difficulty
<img width="1592" height="931" alt="screen_cap4" src="https://github.com/user-attachments/assets/ab9843ea-bdfb-4ee2-8c21-3d4cc5219ec7" />
From the settings menu, you can adjust the maximum number of enemies that can appear on the screen at one time. This allows you to tailor the game's difficulty to your preference. Want a more frantic experience? Crank up the enemy count\!

