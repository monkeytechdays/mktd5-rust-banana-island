MKTD Island Rust Player
=======================

## Goal

Your goal is to develop an AI which must grab most bananas !

All game protocol has been implemented, so just update `src/ai.rs` file. Type alias `ai::AI` is here to help you switch between many implementations.

**Tips**
You may modify `model` module but engine is developed as it is safe to store data into your AI.

## Configuration

Game requires three things:

* Mediator URI (`mediator_uri`): in order to connect game engine.
* Player name (`player_name`): in order to identify. Reuse same player name to reconnect to existing game.
* Player endpoint (`address` + `port`): in order to mediator being able to send game data.

Two first defaults can be found into `Rocket.toml` file. Laters defaults to `localhost:8000`.

Values can be modified into `Rocket.toml` file, but also for a single using `ROCKET_<VARIABLE_NAME>` environment variables.

For example, in order to launch a second player from same code:

```bash
ROCKET_PORT=8001 ROCKET_PLAYER_NAME="Second player" cargo run
```
