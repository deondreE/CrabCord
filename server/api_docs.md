# CrabCord API Documentation

> Auto-generated from live API responses.

## Table of Contents

- [AUTH](#auth)
- [USER PROFILE](#user-profile)
- [DIRECT MESSAGES](#direct-messages)
- [SERVERS](#servers)
- [CHANNELS](#channels)
- [MESSAGES](#messages)
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
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwNjY5ZjQ3Yy01ZDRlLTQyY2UtYWJkMC0xY2MyMDFmY2IwMzIiLCJ1c2VybmFtZSI6InRlc3R1c2VyIiwiZXhwIjoxNzcxODk0MzMxfQ.md7ZttI6KBEIHSMSIFzYfM27K7NzqwNqt2FruYend68",
  "refresh_token": "7iIg1Wosid6AOMs6X8dUd90W3sA1ZiCGXeD9Ahu75KnPUdDjubukrHqb8Nx1f7lO",
  "user": {
    "id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": "/avatars/0669f47c-5d4e-42ce-abd0-1cc201fcb032.png",
    "status": "dnd",
    "created_at": "2026-02-23T00:17:14.181885Z"
  }
}
```

### `POST /auth/refresh`

Exchange a refresh token for a new JWT and rotated refresh token.

**Request Body:**

```json
{
  "refresh_token": "7iIg1Wosid6AOMs6X8dUd90W3sA1ZiCGXeD9Ahu75KnPUdDjubukrHqb8Nx1f7lO"
}
```

**Response:**

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwNjY5ZjQ3Yy01ZDRlLTQyY2UtYWJkMC0xY2MyMDFmY2IwMzIiLCJ1c2VybmFtZSI6InRlc3R1c2VyIiwiZXhwIjoxNzcxODk0MzMyfQ.qT2x7XUs8MS7z393KV7obEdAUA3dZoE8XElZUO4bXJM",
  "refresh_token": "oAm3hiC3qgGJA2YJXQywZ2EcxUk4NAF29wPpr7Ixdt3g9RmN73HHel1cis7Vc8IJ",
  "user": {
    "id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": "/avatars/0669f47c-5d4e-42ce-abd0-1cc201fcb032.png",
    "status": "dnd",
    "created_at": "2026-02-23T00:17:14.181885Z"
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
  "id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "username": "testuser",
  "email": "test@test.com",
  "avatar_url": "/avatars/0669f47c-5d4e-42ce-abd0-1cc201fcb032.png",
  "status": "dnd",
  "created_at": "2026-02-23T00:17:14.181885Z"
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
  "id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "username": "testuser_updated",
  "email": "test@test.com",
  "avatar_url": "/avatars/0669f47c-5d4e-42ce-abd0-1cc201fcb032.png",
  "status": "dnd",
  "created_at": "2026-02-23T00:17:14.181885Z"
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
  "id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "username": "testuser",
  "email": "test@test.com",
  "avatar_url": "/avatars/0669f47c-5d4e-42ce-abd0-1cc201fcb032.png",
  "status": "dnd",
  "created_at": "2026-02-23T00:17:14.181885Z"
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

### `GET /users/263b951f-5c73-43c6-bfd9-56031ac560b6/status`

Get any user's current presence status. Checks in-memory map first, falls back to database.

**Response:**

```json
{
  "status": "online",
  "user_id": "263b951f-5c73-43c6-bfd9-56031ac560b6"
}
```

### `GET /users/search?username=other`

Search users by username using a case-insensitive partial match. Excludes the requesting user. Returns up to 20 results.

**Response:**

```json
[
  {
    "id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
    "username": "otheruser",
    "email": "other@test.com",
    "avatar_url": null,
    "status": "online",
    "created_at": "2026-02-23T00:17:14.747839Z"
  }
]
```

---

## DIRECT MESSAGES

### `POST /dm/263b951f-5c73-43c6-bfd9-56031ac560b6`

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
  "id": "08b193c8-598a-4989-a6bf-72757835fc7c",
  "sender_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "receiver_id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
  "content": "Hey there!",
  "created_at": "2026-02-23T00:52:18.356842Z",
  "edited_at": null,
  "read_at": null
}
```

### `GET /dm/263b951f-5c73-43c6-bfd9-56031ac560b6`

Get DM conversation history between the current user and another user. Returns up to 50 messages in ascending order.

**Response:**

```json
[
  {
    "id": "08b193c8-598a-4989-a6bf-72757835fc7c",
    "sender_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
    "receiver_id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
    "content": "Hey there!",
    "created_at": "2026-02-23T00:52:18.356842Z",
    "edited_at": null,
    "read_at": null
  }
]
```

### `PATCH /dm/263b951f-5c73-43c6-bfd9-56031ac560b6/08b193c8-598a-4989-a6bf-72757835fc7c`

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
  "id": "08b193c8-598a-4989-a6bf-72757835fc7c",
  "sender_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "receiver_id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
  "content": "Edited DM!",
  "created_at": "2026-02-23T00:52:18.356842Z",
  "edited_at": "2026-02-23T00:52:19.335127Z",
  "read_at": null
}
```

### `POST /dm/0669f47c-5d4e-42ce-abd0-1cc201fcb032/08b193c8-598a-4989-a6bf-72757835fc7c/read`

Mark a DM as read. Only the receiver can mark it. Sets `read_at` timestamp.

**Response:**

```json
{
  "id": "08b193c8-598a-4989-a6bf-72757835fc7c",
  "sender_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "receiver_id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
  "content": "Edited DM!",
  "created_at": "2026-02-23T00:52:18.356842Z",
  "edited_at": "2026-02-23T00:52:19.335127Z",
  "read_at": "2026-02-23T00:52:19.826833Z"
}
```

### `GET /dm`

List all DM conversations for the current user. Returns the latest message per conversation.

**Response:**

```json
[
  {
    "id": "08b193c8-598a-4989-a6bf-72757835fc7c",
    "sender_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
    "receiver_id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
    "content": "Edited DM!",
    "created_at": "2026-02-23T00:52:18.356842Z",
    "edited_at": "2026-02-23T00:52:19.335127Z",
    "read_at": "2026-02-23T00:52:19.826833Z"
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
  "id": "7876729c-630d-44d7-9638-70d51d39a44c",
  "name": "Test Server",
  "owner_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "created_at": "2026-02-23T00:52:21.223795Z"
}
```

### `PATCH /servers/7876729c-630d-44d7-9638-70d51d39a44c`

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
  "id": "7876729c-630d-44d7-9638-70d51d39a44c",
  "name": "Renamed Server",
  "owner_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "created_at": "2026-02-23T00:52:21.223795Z"
}
```

---

## CHANNELS

### `POST /servers/7876729c-630d-44d7-9638-70d51d39a44c/channels`

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
  "id": "c73d0c95-16cd-4bce-ad66-fcee9554526e",
  "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
  "name": "general",
  "created_at": "2026-02-23T00:52:22.670154Z"
}
```

### `GET /servers/7876729c-630d-44d7-9638-70d51d39a44c/channels`

List all channels in a server ordered by creation time.

**Response:**

```json
[
  {
    "id": "2d6152ef-3669-44c9-8906-4dc2b53688f5",
    "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
    "name": "general-chat",
    "created_at": "2026-02-23T00:52:21.223795Z"
  },
  {
    "id": "c73d0c95-16cd-4bce-ad66-fcee9554526e",
    "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
    "name": "general",
    "created_at": "2026-02-23T00:52:22.670154Z"
  },
  {
    "id": "f659c37f-177c-4ddf-b138-d32ead392d0c",
    "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
    "name": "off-topic",
    "created_at": "2026-02-23T00:52:23.382502Z"
  }
]
```

### `PATCH /servers/7876729c-630d-44d7-9638-70d51d39a44c/channels/c73d0c95-16cd-4bce-ad66-fcee9554526e`

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
  "id": "c73d0c95-16cd-4bce-ad66-fcee9554526e",
  "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
  "name": "general-updated",
  "created_at": "2026-02-23T00:52:22.670154Z"
}
```

---

## MESSAGES

### `POST /channels/c73d0c95-16cd-4bce-ad66-fcee9554526e/messages`

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
  "id": "29b6ca81-7aa2-4bf6-a68d-d15fd32616c4",
  "channel_id": "c73d0c95-16cd-4bce-ad66-fcee9554526e",
  "user_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "username": "testuser",
  "content": "Hello world!",
  "created_at": "2026-02-23T00:52:24.331676Z",
  "edited_at": null
}
```

### `GET /channels/c73d0c95-16cd-4bce-ad66-fcee9554526e/messages`

Retrieve the last 50 messages in a channel, ordered oldest first. Includes the author's username.

**Response:**

```json
[
  {
    "id": "29b6ca81-7aa2-4bf6-a68d-d15fd32616c4",
    "channel_id": "c73d0c95-16cd-4bce-ad66-fcee9554526e",
    "user_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
    "username": "testuser",
    "content": "Hello world!",
    "created_at": "2026-02-23T00:52:24.331676Z",
    "edited_at": null
  }
]
```

### `PATCH /channels/c73d0c95-16cd-4bce-ad66-fcee9554526e/messages/29b6ca81-7aa2-4bf6-a68d-d15fd32616c4`

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
  "id": "29b6ca81-7aa2-4bf6-a68d-d15fd32616c4",
  "channel_id": "c73d0c95-16cd-4bce-ad66-fcee9554526e",
  "user_id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "username": "testuser",
  "content": "Edited!",
  "created_at": "2026-02-23T00:52:24.331676Z",
  "edited_at": "2026-02-23T00:52:25.510782Z"
}
```

---

## ROLES

### `POST /servers/7876729c-630d-44d7-9638-70d51d39a44c/roles`

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
  "id": "562cf02c-735f-4221-871c-e9b8ebf63c4b",
  "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
  "name": "Moderator",
  "permissions": 46,
  "created_at": "2026-02-23T00:52:26.921132Z"
}
```

### `GET /servers/7876729c-630d-44d7-9638-70d51d39a44c/roles`

List all roles in a server ordered by creation time.

**Response:**

```json
[
  {
    "id": "562cf02c-735f-4221-871c-e9b8ebf63c4b",
    "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
    "name": "Moderator",
    "permissions": 46,
    "created_at": "2026-02-23T00:52:26.921132Z"
  },
  {
    "id": "b0684473-d7fe-48f9-8676-12d1275d2d2f",
    "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
    "name": "Admin",
    "permissions": 128,
    "created_at": "2026-02-23T00:52:27.158671Z"
  }
]
```

---

## INVITES & MEMBERS

### `POST /servers/7876729c-630d-44d7-9638-70d51d39a44c/invites`

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
  "id": "5e274c7a-203a-434f-ae6c-0c6a2dde3086",
  "code": "CZH8RYkc",
  "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
  "created_by": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "max_uses": 5,
  "uses": 0,
  "expires_at": null,
  "created_at": "2026-02-23T00:52:27.870629Z"
}
```

### `GET /invites/CZH8RYkc`

Get invite metadata by code. Does not require authentication and does not consume a use.

**Response:**

```json
{
  "id": "5e274c7a-203a-434f-ae6c-0c6a2dde3086",
  "code": "CZH8RYkc",
  "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
  "created_by": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
  "max_uses": 5,
  "uses": 0,
  "expires_at": null,
  "created_at": "2026-02-23T00:52:27.870629Z"
}
```

### `POST /invites/CZH8RYkc/join`

Use an invite code to join a server. Validates expiry and max uses. Returns the new `ServerMember` record.

**Response:**

```json
{
  "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
  "user_id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
  "joined_at": "2026-02-23T00:52:28.856955Z"
}
```

### `GET /servers/7876729c-630d-44d7-9638-70d51d39a44c/members`

List all members of a server with their full profile data, ordered by join time.

**Response:**

```json
[
  {
    "id": "0669f47c-5d4e-42ce-abd0-1cc201fcb032",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": "/avatars/0669f47c-5d4e-42ce-abd0-1cc201fcb032.png",
    "status": "dnd",
    "created_at": "2026-02-23T00:17:14.181885Z"
  },
  {
    "id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
    "username": "otheruser",
    "email": "other@test.com",
    "avatar_url": null,
    "status": "online",
    "created_at": "2026-02-23T00:17:14.747839Z"
  }
]
```

### `POST /servers/7876729c-630d-44d7-9638-70d51d39a44c/roles/assign`

Assign a role to a server member. Requires `MANAGE_ROLES` permission. The user must already be a member.

**Request Body:**

```json
{
  "user_id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
  "role_id": "562cf02c-735f-4221-871c-e9b8ebf63c4b"
}
```

**Response:**

```json
{
  "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
  "user_id": "263b951f-5c73-43c6-bfd9-56031ac560b6",
  "role_id": "562cf02c-735f-4221-871c-e9b8ebf63c4b",
  "assigned_at": "2026-02-23T00:52:29.556656Z"
}
```

### `GET /servers/7876729c-630d-44d7-9638-70d51d39a44c/members/263b951f-5c73-43c6-bfd9-56031ac560b6/roles`

List all roles currently held by a specific user in a server.

**Response:**

```json
[
  {
    "id": "562cf02c-735f-4221-871c-e9b8ebf63c4b",
    "server_id": "7876729c-630d-44d7-9638-70d51d39a44c",
    "name": "Moderator",
    "permissions": 46,
    "created_at": "2026-02-23T00:52:26.921132Z"
  }
]
```

---

