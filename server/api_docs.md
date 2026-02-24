# CrabCord API Documentation

> Auto-generated from live API responses.

## Table of Contents

- [AUTH](#auth)
- [USER PROFILE](#user-profile)
- [DIRECT MESSAGES](#direct-messages)
- [SERVERS](#servers)
- [CHANNELS](#channels)
- [MESSAGES](#messages)
- [REACTIONS](#reactions)
- [ROLES](#roles)
- [INVITES & MEMBERS](#invites--members)

---

## AUTH

### `POST /auth/login`

Authenticate with email and password. Returns a JWT, refresh token, and user object.

**Request Body:**

```json
{
  "email": "test@test.com",
  "password": "password123"
}
```

**Response:**

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIyNDZmZmViMi04ODk0LTQyYjEtYjQ2My0wYjEwMGRmNWYxMmIiLCJ1c2VybmFtZSI6InRlc3R1c2VyIiwiZXhwIjoxNzcxOTkzNDkxfQ.wRBKzoc5IcQq70opR0ALOS_X8fq2dDrEsibf8ahgqs0",
  "refresh_token": "yj7rslqZyU7bj1sysYVQbFpftuOVUo8dkn1foXQDrgiktJkNpl68l36tkC28iyAt",
  "user": {
    "id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": "/avatars/246ffeb2-8894-42b1-b463-0b100df5f12b.png",
    "status": "dnd",
    "created_at": "2026-02-24T04:22:26.063819Z"
  }
}
```

### `POST /auth/refresh`

Exchange a refresh token for a new JWT and rotated refresh token.

**Request Body:**

```json
{
  "refresh_token": "yj7rslqZyU7bj1sysYVQbFpftuOVUo8dkn1foXQDrgiktJkNpl68l36tkC28iyAt"
}
```

**Response:**

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIyNDZmZmViMi04ODk0LTQyYjEtYjQ2My0wYjEwMGRmNWYxMmIiLCJ1c2VybmFtZSI6InRlc3R1c2VyIiwiZXhwIjoxNzcxOTkzNDkyfQ.bI0Xll2FIcCv1tXqN9X9Tu-hZWGYqmK4pbnB2Cw5XJQ",
  "refresh_token": "tRS8guDTZhjDFJHAdRf5qoFkBXUJIdl69o8V2vBQMhxOPC9YJ4WXQFFahv41wBPD",
  "user": {
    "id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": "/avatars/246ffeb2-8894-42b1-b463-0b100df5f12b.png",
    "status": "dnd",
    "created_at": "2026-02-24T04:22:26.063819Z"
  }
}
```

---

## USER PROFILE

### `GET /users/me`

Returns the currently authenticated user's profile.

**Response:**

```json
{
  "id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "username": "testuser",
  "email": "test@test.com",
  "avatar_url": "/avatars/246ffeb2-8894-42b1-b463-0b100df5f12b.png",
  "status": "dnd",
  "created_at": "2026-02-24T04:22:26.063819Z"
}
```

### `PATCH /users/me`

Update the current user's username, email, or password. All fields are optional.

**Request Body:**

```json
{
  "username": "testuser_updated"
}
```

**Response:**

```json
{
  "id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "username": "testuser_updated",
  "email": "test@test.com",
  "avatar_url": "/avatars/246ffeb2-8894-42b1-b463-0b100df5f12b.png",
  "status": "dnd",
  "created_at": "2026-02-24T04:22:26.063819Z"
}
```

### `POST /users/me/avatar`

Upload a profile picture. Send as multipart/form-data with field name `avatar`. Max 5MB.

**Request Body:**

```json
{
  "avatar": "<PNG file (multipart/form-data, field name: avatar)>"
}
```

**Response:**

```json
{
  "id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "username": "testuser",
  "email": "test@test.com",
  "avatar_url": "/avatars/246ffeb2-8894-42b1-b463-0b100df5f12b.png",
  "status": "dnd",
  "created_at": "2026-02-24T04:22:26.063819Z"
}
```

### `PATCH /users/me/status`

Update the current user's presence status. Valid values: `online`, `idle`, `dnd`, `offline`.

**Request Body:**

```json
{
  "status": "online"
}
```

**Response:**

```json
{
  "status": "online"
}
```

### `GET /users/1fe26044-fd81-4ca4-aaaf-bf247e236816/status`

Get any user's current presence status. Checks in-memory map first, falls back to database.

**Response:**

```json
{
  "status": "online",
  "user_id": "1fe26044-fd81-4ca4-aaaf-bf247e236816"
}
```

### `GET /users/search?username=other`

Search users by username using a case-insensitive partial match. Excludes the requesting user. Returns up to 20 results.

**Response:**

```json
[
  {
    "id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
    "username": "otheruser",
    "email": "other@test.com",
    "avatar_url": null,
    "status": "online",
    "created_at": "2026-02-24T04:22:26.621481Z"
  }
]
```

---

## DIRECT MESSAGES

### `POST /dm/1fe26044-fd81-4ca4-aaaf-bf247e236816`

Send a direct message to another user by their ID. Cannot DM yourself.

**Request Body:**

```json
{
  "content": "Hey there!"
}
```

**Response:**

```json
{
  "id": "f433e968-d08a-46b4-9751-9d96c767a30f",
  "sender_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "receiver_id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
  "content": "Hey there!",
  "created_at": "2026-02-24T04:24:58.440220Z",
  "edited_at": null,
  "read_at": null
}
```

### `GET /dm/1fe26044-fd81-4ca4-aaaf-bf247e236816`

Get DM conversation history between the current user and another user. Returns up to 50 messages in ascending order.

**Response:**

```json
[
  {
    "id": "f433e968-d08a-46b4-9751-9d96c767a30f",
    "sender_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
    "receiver_id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
    "content": "Hey there!",
    "created_at": "2026-02-24T04:24:58.440220Z",
    "edited_at": null,
    "read_at": null
  }
]
```

### `PATCH /dm/1fe26044-fd81-4ca4-aaaf-bf247e236816/f433e968-d08a-46b4-9751-9d96c767a30f`

Edit a direct message. Only the original sender can edit. Sets `edited_at` timestamp.

**Request Body:**

```json
{
  "content": "Edited DM!"
}
```

**Response:**

```json
{
  "id": "f433e968-d08a-46b4-9751-9d96c767a30f",
  "sender_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "receiver_id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
  "content": "Edited DM!",
  "created_at": "2026-02-24T04:24:58.440220Z",
  "edited_at": "2026-02-24T04:24:59.423251Z",
  "read_at": null
}
```

### `POST /dm/246ffeb2-8894-42b1-b463-0b100df5f12b/f433e968-d08a-46b4-9751-9d96c767a30f/read`

Mark a DM as read. Only the receiver can mark it. Sets `read_at` timestamp.

**Response:**

```json
{
  "id": "f433e968-d08a-46b4-9751-9d96c767a30f",
  "sender_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "receiver_id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
  "content": "Edited DM!",
  "created_at": "2026-02-24T04:24:58.440220Z",
  "edited_at": "2026-02-24T04:24:59.423251Z",
  "read_at": "2026-02-24T04:24:59.935221Z"
}
```

### `GET /dm`

List all DM conversations for the current user. Returns the latest message per conversation.

**Response:**

```json
[
  {
    "id": "f433e968-d08a-46b4-9751-9d96c767a30f",
    "sender_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
    "receiver_id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
    "content": "Edited DM!",
    "created_at": "2026-02-24T04:24:58.440220Z",
    "edited_at": "2026-02-24T04:24:59.423251Z",
    "read_at": "2026-02-24T04:24:59.935221Z"
  }
]
```

---

## SERVERS

### `POST /servers`

Create a new server. The creator is automatically added as the owner and first member.

**Request Body:**

```json
{
  "name": "Test Server"
}
```

**Response:**

```json
{
  "id": "87232da3-f3ca-47d1-8185-9b7058854463",
  "name": "Test Server",
  "owner_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "created_at": "2026-02-24T04:25:01.362623Z"
}
```

### `PATCH /servers/87232da3-f3ca-47d1-8185-9b7058854463`

Update a server's name. Only the server owner can do this.

**Request Body:**

```json
{
  "name": "Renamed Server"
}
```

**Response:**

```json
{
  "id": "87232da3-f3ca-47d1-8185-9b7058854463",
  "name": "Renamed Server",
  "owner_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "created_at": "2026-02-24T04:25:01.362623Z"
}
```

---

## CHANNELS

### `POST /servers/87232da3-f3ca-47d1-8185-9b7058854463/channels`

Create a channel in a server. Requires `MANAGE_CHANNELS` permission. Channel names must be unique per server.

**Request Body:**

```json
{
  "name": "general"
}
```

**Response:**

```json
{
  "id": "a71f131a-4b79-471a-b843-459eee6e2b2b",
  "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
  "name": "general",
  "created_at": "2026-02-24T04:25:02.856857Z"
}
```

### `GET /servers/87232da3-f3ca-47d1-8185-9b7058854463/channels`

List all channels in a server ordered by creation time.

**Response:**

```json
[
  {
    "id": "eb69feaf-ee57-4366-afc7-207391d39bbf",
    "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
    "name": "general-chat",
    "created_at": "2026-02-24T04:25:01.362623Z"
  },
  {
    "id": "a71f131a-4b79-471a-b843-459eee6e2b2b",
    "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
    "name": "general",
    "created_at": "2026-02-24T04:25:02.856857Z"
  },
  {
    "id": "18c776cc-2806-4aba-950b-0ce958b84d73",
    "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
    "name": "off-topic",
    "created_at": "2026-02-24T04:25:03.590957Z"
  }
]
```

### `PATCH /servers/87232da3-f3ca-47d1-8185-9b7058854463/channels/a71f131a-4b79-471a-b843-459eee6e2b2b`

Rename a channel. Only the server owner can do this.

**Request Body:**

```json
{
  "name": "general-updated"
}
```

**Response:**

```json
{
  "id": "a71f131a-4b79-471a-b843-459eee6e2b2b",
  "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
  "name": "general-updated",
  "created_at": "2026-02-24T04:25:02.856857Z"
}
```

---

## MESSAGES

### `POST /channels/a71f131a-4b79-471a-b843-459eee6e2b2b/messages`

Post a message to a channel. Content must be between 1 and 2000 characters.

**Request Body:**

```json
{
  "content": "Hello world!"
}
```

**Response:**

```json
{
  "id": "4a5906cf-3ac2-4602-8e32-1063c763df77",
  "channel_id": "a71f131a-4b79-471a-b843-459eee6e2b2b",
  "user_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "username": "testuser",
  "content": "Hello world!",
  "reactions": [],
  "created_at": "2026-02-24T04:25:04.585517Z",
  "edited_at": null
}
```

### `GET /channels/a71f131a-4b79-471a-b843-459eee6e2b2b/messages`

Retrieve the last 50 messages in a channel, ordered oldest first. Includes the author's username and reactions.

**Response:**

```json
[
  {
    "id": "4a5906cf-3ac2-4602-8e32-1063c763df77",
    "channel_id": "a71f131a-4b79-471a-b843-459eee6e2b2b",
    "user_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
    "username": "testuser",
    "content": "Hello world!",
    "reactions": [],
    "created_at": "2026-02-24T04:25:04.585517Z",
    "edited_at": null
  }
]
```

### `PATCH /channels/a71f131a-4b79-471a-b843-459eee6e2b2b/messages/4a5906cf-3ac2-4602-8e32-1063c763df77`

Edit a message's content. Only the original author can edit. Sets `edited_at` timestamp.

**Request Body:**

```json
{
  "content": "Edited!"
}
```

**Response:**

```json
{
  "id": "4a5906cf-3ac2-4602-8e32-1063c763df77",
  "channel_id": "a71f131a-4b79-471a-b843-459eee6e2b2b",
  "user_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "username": "testuser",
  "content": "Edited!",
  "reactions": [],
  "created_at": "2026-02-24T04:25:04.585517Z",
  "edited_at": "2026-02-24T04:25:05.873071Z"
}
```

---

## REACTIONS

### `PUT /messages/4a5906cf-3ac2-4602-8e32-1063c763df77/reactions/👍`

Toggle a reaction ON for the current user. Idempotent — adding the same reaction twice has no effect. Returns the full updated message.

**Response:**

```json
{
  "id": "4a5906cf-3ac2-4602-8e32-1063c763df77",
  "channel_id": "a71f131a-4b79-471a-b843-459eee6e2b2b",
  "user_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "username": "testuser",
  "content": "Edited!",
  "reactions": [
    {
      "emoji_id": "\ud83d\udc4d",
      "count": 1,
      "user_ids": [
        "246ffeb2-8894-42b1-b463-0b100df5f12b"
      ]
    }
  ],
  "created_at": "2026-02-24T04:25:04.585517Z",
  "edited_at": "2026-02-24T04:25:05.873071Z"
}
```

### `PUT /messages/4a5906cf-3ac2-4602-8e32-1063c763df77/reactions/🔥`

Multiple distinct emojis can be added to the same message.

**Response:**

```json
{
  "id": "4a5906cf-3ac2-4602-8e32-1063c763df77",
  "channel_id": "a71f131a-4b79-471a-b843-459eee6e2b2b",
  "user_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "username": "testuser",
  "content": "Edited!",
  "reactions": [
    {
      "emoji_id": "\ud83d\udc4d",
      "count": 2,
      "user_ids": [
        "246ffeb2-8894-42b1-b463-0b100df5f12b",
        "1fe26044-fd81-4ca4-aaaf-bf247e236816"
      ]
    },
    {
      "emoji_id": "\ud83d\udd25",
      "count": 1,
      "user_ids": [
        "246ffeb2-8894-42b1-b463-0b100df5f12b"
      ]
    }
  ],
  "created_at": "2026-02-24T04:25:04.585517Z",
  "edited_at": "2026-02-24T04:25:05.873071Z"
}
```

### `DELETE /messages/4a5906cf-3ac2-4602-8e32-1063c763df77/reactions/🔥`

Remove a reaction for the current user. Returns 404 if the user hadn't added that reaction. Returns the full updated message.

**Response:**

```json
{
  "id": "4a5906cf-3ac2-4602-8e32-1063c763df77",
  "channel_id": "a71f131a-4b79-471a-b843-459eee6e2b2b",
  "user_id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "username": "testuser",
  "content": "Edited!",
  "reactions": [
    {
      "emoji_id": "\ud83d\udc4d",
      "count": 2,
      "user_ids": [
        "246ffeb2-8894-42b1-b463-0b100df5f12b",
        "1fe26044-fd81-4ca4-aaaf-bf247e236816"
      ]
    }
  ],
  "created_at": "2026-02-24T04:25:04.585517Z",
  "edited_at": "2026-02-24T04:25:05.873071Z"
}
```

---

## ROLES

### `POST /servers/87232da3-f3ca-47d1-8185-9b7058854463/roles`

Create a role in a server. `permissions` is a bitmask. Requires `MANAGE_ROLES` permission. Permission bits: VIEW_CHANNELS=1, SEND_MESSAGES=2, MANAGE_MESSAGES=4, MANAGE_CHANNELS=8, MANAGE_ROLES=16, KICK_MEMBERS=32, BAN_MEMBERS=64, ADMINISTRATOR=128.

**Request Body:**

```json
{
  "name": "Moderator",
  "permissions": 46
}
```

**Response:**

```json
{
  "id": "57508b48-257f-4f25-bed8-b4bd2b976bea",
  "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
  "name": "Moderator",
  "permissions": 46,
  "created_at": "2026-02-24T04:25:09.539600Z"
}
```

### `GET /servers/87232da3-f3ca-47d1-8185-9b7058854463/roles`

List all roles in a server ordered by creation time.

**Response:**

```json
[
  {
    "id": "57508b48-257f-4f25-bed8-b4bd2b976bea",
    "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
    "name": "Moderator",
    "permissions": 46,
    "created_at": "2026-02-24T04:25:09.539600Z"
  },
  {
    "id": "cdc484cd-be8e-4909-a233-084f7ee7a8b1",
    "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
    "name": "Admin",
    "permissions": 128,
    "created_at": "2026-02-24T04:25:09.771984Z"
  }
]
```

---

## INVITES & MEMBERS

### `POST /servers/87232da3-f3ca-47d1-8185-9b7058854463/invites`

Create an invite link for a server. `max_uses` and `expires_at` are optional. Only server members can create invites.

**Request Body:**

```json
{
  "max_uses": 5
}
```

**Response:**

```json
{
  "id": "7062650a-36a3-4942-87e6-82fb81263ac3",
  "code": "y5eXBKyX",
  "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
  "created_by": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "max_uses": 5,
  "uses": 0,
  "expires_at": null,
  "created_at": "2026-02-24T04:25:10.515960Z"
}
```

### `GET /invites/y5eXBKyX`

Get invite metadata by code. Does not require authentication and does not consume a use.

**Response:**

```json
{
  "id": "7062650a-36a3-4942-87e6-82fb81263ac3",
  "code": "y5eXBKyX",
  "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
  "created_by": "246ffeb2-8894-42b1-b463-0b100df5f12b",
  "max_uses": 5,
  "uses": 0,
  "expires_at": null,
  "created_at": "2026-02-24T04:25:10.515960Z"
}
```

### `POST /invites/y5eXBKyX/join`

Use an invite code to join a server. Validates expiry and max uses. Returns the new `ServerMember` record.

**Response:**

```json
{
  "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
  "user_id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
  "joined_at": "2026-02-24T04:25:11.495268Z"
}
```

### `GET /servers/87232da3-f3ca-47d1-8185-9b7058854463/members`

List all members of a server with their full profile data, ordered by join time.

**Response:**

```json
[
  {
    "id": "246ffeb2-8894-42b1-b463-0b100df5f12b",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": "/avatars/246ffeb2-8894-42b1-b463-0b100df5f12b.png",
    "status": "dnd",
    "created_at": "2026-02-24T04:22:26.063819Z"
  },
  {
    "id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
    "username": "otheruser",
    "email": "other@test.com",
    "avatar_url": null,
    "status": "online",
    "created_at": "2026-02-24T04:22:26.621481Z"
  }
]
```

### `POST /servers/87232da3-f3ca-47d1-8185-9b7058854463/roles/assign`

Assign a role to a server member. Requires `MANAGE_ROLES` permission. The user must already be a member.

**Request Body:**

```json
{
  "user_id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
  "role_id": "57508b48-257f-4f25-bed8-b4bd2b976bea"
}
```

**Response:**

```json
{
  "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
  "user_id": "1fe26044-fd81-4ca4-aaaf-bf247e236816",
  "role_id": "57508b48-257f-4f25-bed8-b4bd2b976bea",
  "assigned_at": "2026-02-24T04:25:12.219092Z"
}
```

### `GET /servers/87232da3-f3ca-47d1-8185-9b7058854463/members/1fe26044-fd81-4ca4-aaaf-bf247e236816/roles`

List all roles currently held by a specific user in a server.

**Response:**

```json
[
  {
    "id": "57508b48-257f-4f25-bed8-b4bd2b976bea",
    "server_id": "87232da3-f3ca-47d1-8185-9b7058854463",
    "name": "Moderator",
    "permissions": 46,
    "created_at": "2026-02-24T04:25:09.539600Z"
  }
]
```

---

