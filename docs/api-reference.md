# REST API Reference

The poker server exposes a RESTful JSON API on port **8084** (configurable via `PORT` env var).

---

## Health

### `GET /health`

Health check endpoint.

**Response:**

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

---

## Rulesets

### `GET /api/rulesets`

List all available poker variants.

**Response:**

```json
[
  {
    "name": "texas_holdem",
    "display_name": "Texas Hold'em",
    "description": "Classic poker variant with 2 hole cards and 5 community cards.",
    "min_players": 2,
    "max_players": 10,
    "hole_cards": 2,
    "community_cards": 5,
    "betting_structure": "NoLimit"
  }
]
```

---

## Games

### `POST /api/games`

Create a new poker game.

**Request body:**

```json
{
  "variant": "texas_holdem",
  "small_blind": 10,
  "big_blind": 20
}
```

All fields are optional. Defaults: variant=`texas_holdem`, small_blind=`10`, big_blind=`20`.

**Response:** Full game state (see below).

---

### `GET /api/games`

List all active games.

**Response:**

```json
[
  {
    "id": "550e8400-...",
    "variant": "texas_holdem",
    "phase": "PreFlop",
    "player_count": 4,
    "pot": 30
  }
]
```

---

### `GET /api/games/{id}`

Get the full state of a game.

**Response:**

```json
{
  "id": "550e8400-...",
  "variant": "texas_holdem",
  "phase": "Flop",
  "pot": 60,
  "community_cards": [
    { "rank": "K", "suit": "spades" },
    { "rank": "8", "suit": "hearts" },
    { "rank": "3", "suit": "clubs" }
  ],
  "players": [
    {
      "id": "player-uuid-1",
      "name": "Alice",
      "chips": 980,
      "status": "Active",
      "total_bet": 20,
      "hole_cards": null
    }
  ],
  "current_player_id": "player-uuid-1",
  "available_actions": ["fold", "check", "raise", "all_in"],
  "dealer_position": 0,
  "hand_number": 1,
  "small_blind": 10,
  "big_blind": 20,
  "showdown": []
}
```

**Note:** Hole cards are only visible in the player-specific view (`/player/{id}`), or after showdown.

---

### `DELETE /api/games/{id}`

Delete a game.

**Response:**

```json
{ "deleted": "game-uuid" }
```

---

## Player Management

### `POST /api/games/{id}/join`

Join a game.

**Request body:**

```json
{
  "name": "Alice",
  "chips": 1000
}
```

`chips` is optional (default: 1000).

---

### `POST /api/games/{id}/leave`

Leave a game.

**Request body:**

```json
{
  "player_id": "player-uuid"
}
```

---

## Game Flow

### `POST /api/games/{id}/start`

Start a new hand. Requires at least the minimum number of players for the variant.

---

### `POST /api/games/{id}/action`

Submit a player action.

**Request body:**

```json
{
  "player_id": "player-uuid",
  "action": "raise",
  "amount": 50
}
```

**Supported actions:**

| Action | Fields | Description |
|--------|--------|-------------|
| `fold` | — | Fold your hand |
| `check` | — | Check (no bet) |
| `call` | — | Match the current bet |
| `raise` | `amount` | Raise to the specified amount |
| `all_in` | — | Go all-in |
| `discard` | `discard_indices` | Discard cards (Five-Card Draw only) |

---

### `GET /api/games/{id}/player/{player_id}`

Get the game state from a specific player's perspective (shows their hole cards).

**Response:**

```json
{
  "game": { "..." },
  "your_hole_cards": [
    { "rank": "A", "suit": "spades" },
    { "rank": "K", "suit": "hearts" }
  ]
}
```

---

### `GET /api/games/{id}/history`

Get the action history for the current hand.

**Response:**

```json
{
  "id": "game-uuid",
  "hand_number": 1,
  "actions": [
    {
      "player_id": "player-uuid",
      "action": "Call",
      "phase": "PreFlop"
    }
  ]
}
```

---

## Error Responses

All errors return a consistent JSON format:

```json
{
  "error": "error_type",
  "message": "Human-readable description"
}
```

| HTTP Status | Error Type | Description |
|-------------|-----------|-------------|
| 400 | `invalid_action` | Invalid action for current state |
| 404 | `game_not_found` | Game ID not found |
| 404 | `player_not_found` | Player ID not found |
| 500 | `internal_error` | Unexpected server error |
