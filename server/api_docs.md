# CrabCord API Documentation

> Auto-generated from live API responses.

## Table of Contents

- [AUTH](#auth)
- [USER PROFILE](#user-profile)
- [DIRECT MESSAGES](#direct-messages)
- [SERVERS](#servers)
- [CHANNELS](#channels)
- [VOICE CHANNELS](#voice-channels)
- [MESSAGES](#messages)
- [REACTIONS](#reactions)
- [ROLES](#roles)
- [INVITES & MEMBERS](#invites--members)

---

## AUTH

### `POST /auth/register`

Register a new user. Returns a JWT, refresh token, and user object.

**Request Body:**

```json
{
  "username": "testuser",
  "email": "test@test.com",
  "password": "password123"
}
```

**Response:**

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJlODAzZTA0Ny0yZTI5LTRjMzgtODY0My0xODI3MThjYWFhMDgiLCJ1c2VybmFtZSI6InRlc3R1c2VyIiwiZXhwIjoxNzcyMDU2MDg2fQ.k5xb0Gw3VuF_wse_RcPZQJh66R3v8mpGNpNIdLBch9k",
  "refresh_token": "ZuP292c0XdLBdA89DustwU30OGjOHGbt5tVw9p8sG9Xt0lrYCe7Gw6zaVFp8jO8C",
  "user": {
    "id": "e803e047-2e29-4c38-8643-182718caaa08",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": null,
    "status": "online",
    "created_at": "2026-02-24T21:48:06.701994Z"
  }
}
```

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
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJlODAzZTA0Ny0yZTI5LTRjMzgtODY0My0xODI3MThjYWFhMDgiLCJ1c2VybmFtZSI6InRlc3R1c2VyIiwiZXhwIjoxNzcyMDU2MDg4fQ.zWGZRVXQZnfl5U1egUd_GT71buhq1BQp7iMLQErg_24",
  "refresh_token": "P7lKZm8DE1NnA1ZDZQEhh4fYrI6Vp53EXaDNBfKkZWq3XipsopiGKUfarIekIJj5",
  "user": {
    "id": "e803e047-2e29-4c38-8643-182718caaa08",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": null,
    "status": "online",
    "created_at": "2026-02-24T21:48:06.701994Z"
  }
}
```

### `POST /auth/refresh`

Exchange a refresh token for a new JWT and rotated refresh token.

**Request Body:**

```json
{
  "refresh_token": "P7lKZm8DE1NnA1ZDZQEhh4fYrI6Vp53EXaDNBfKkZWq3XipsopiGKUfarIekIJj5"
}
```

**Response:**

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJlODAzZTA0Ny0yZTI5LTRjMzgtODY0My0xODI3MThjYWFhMDgiLCJ1c2VybmFtZSI6InRlc3R1c2VyIiwiZXhwIjoxNzcyMDU2MDg5fQ.3r_ShneQhhaB7iTG87aTFMyWMMHWmOWgd4Up71J992M",
  "refresh_token": "yp0NHKG0gYCNREgv7zbgds1uZ9xYpMO5pfjVzbooFtV8iOGOP8jcLqB8yPPpKSKt",
  "user": {
    "id": "e803e047-2e29-4c38-8643-182718caaa08",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": null,
    "status": "online",
    "created_at": "2026-02-24T21:48:06.701994Z"
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
  "id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser",
  "email": "test@test.com",
  "avatar_url": null,
  "status": "online",
  "created_at": "2026-02-24T21:48:06.701994Z"
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
  "id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser_updated",
  "email": "test@test.com",
  "avatar_url": null,
  "status": "online",
  "created_at": "2026-02-24T21:48:06.701994Z"
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
  "id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser",
  "email": "test@test.com",
  "avatar_url": "/avatars/e803e047-2e29-4c38-8643-182718caaa08.png",
  "status": "online",
  "created_at": "2026-02-24T21:48:06.701994Z"
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

### `GET /users/58961f99-4163-47a3-a64d-df86ac11bd1c/status`

Get any user's current presence status. Checks in-memory map first, falls back to database.

**Response:**

```json
{
  "status": "online",
  "user_id": "58961f99-4163-47a3-a64d-df86ac11bd1c"
}
```

### `GET /users/search?username=other`

Search users by username using a case-insensitive partial match. Excludes the requesting user. Returns up to 20 results.

**Response:**

```json
[
  {
    "id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
    "username": "otheruser",
    "email": "other@test.com",
    "avatar_url": null,
    "status": "online",
    "created_at": "2026-02-24T21:48:07.315844Z"
  }
]
```

---

## DIRECT MESSAGES

### `POST /dm/58961f99-4163-47a3-a64d-df86ac11bd1c`

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
  "id": "42bdecdd-4cb2-4b9b-8809-4107a13f425d",
  "sender_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "receiver_id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
  "content": "Hey there!",
  "created_at": "2026-02-24T21:48:15.617248Z",
  "edited_at": null,
  "read_at": null
}
```

### `GET /dm/58961f99-4163-47a3-a64d-df86ac11bd1c`

Get DM conversation history between the current user and another user. Returns up to 50 messages in ascending order.

**Response:**

```json
[
  {
    "id": "42bdecdd-4cb2-4b9b-8809-4107a13f425d",
    "sender_id": "e803e047-2e29-4c38-8643-182718caaa08",
    "receiver_id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
    "content": "Hey there!",
    "created_at": "2026-02-24T21:48:15.617248Z",
    "edited_at": null,
    "read_at": null
  }
]
```

### `PATCH /dm/58961f99-4163-47a3-a64d-df86ac11bd1c/42bdecdd-4cb2-4b9b-8809-4107a13f425d`

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
  "id": "42bdecdd-4cb2-4b9b-8809-4107a13f425d",
  "sender_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "receiver_id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
  "content": "Edited DM!",
  "created_at": "2026-02-24T21:48:15.617248Z",
  "edited_at": "2026-02-24T21:48:16.643084Z",
  "read_at": null
}
```

### `POST /dm/e803e047-2e29-4c38-8643-182718caaa08/42bdecdd-4cb2-4b9b-8809-4107a13f425d/read`

Mark a DM as read. Only the receiver can mark it. Sets `read_at` timestamp.

**Response:**

```json
{
  "id": "42bdecdd-4cb2-4b9b-8809-4107a13f425d",
  "sender_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "receiver_id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
  "content": "Edited DM!",
  "created_at": "2026-02-24T21:48:15.617248Z",
  "edited_at": "2026-02-24T21:48:16.643084Z",
  "read_at": "2026-02-24T21:48:17.158905Z"
}
```

### `GET /dm`

List all DM conversations for the current user. Returns the latest message per conversation.

**Response:**

```json
[
  {
    "id": "42bdecdd-4cb2-4b9b-8809-4107a13f425d",
    "sender_id": "e803e047-2e29-4c38-8643-182718caaa08",
    "receiver_id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
    "content": "Edited DM!",
    "created_at": "2026-02-24T21:48:15.617248Z",
    "edited_at": "2026-02-24T21:48:16.643084Z",
    "read_at": "2026-02-24T21:48:17.158905Z"
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
  "id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "name": "Test Server",
  "owner_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "created_at": "2026-02-24T21:48:18.672818Z"
}
```

### `PATCH /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d`

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
  "id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "name": "Renamed Server",
  "owner_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "created_at": "2026-02-24T21:48:18.672818Z"
}
```

---

## CHANNELS

### `POST /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/channels`

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
  "id": "352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b",
  "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "name": "general",
  "created_at": "2026-02-24T21:48:20.242852Z"
}
```

### `GET /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/channels`

List all channels in a server ordered by creation time.

**Response:**

```json
[
  {
    "id": "d72c1ab3-06d5-4f2d-a3da-ebdcc9bdde59",
    "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
    "name": "general-chat",
    "created_at": "2026-02-24T21:48:18.672818Z"
  },
  {
    "id": "352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b",
    "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
    "name": "general",
    "created_at": "2026-02-24T21:48:20.242852Z"
  },
  {
    "id": "3c1c2e49-c5a9-4101-aebf-5e902c007eaf",
    "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
    "name": "off-topic",
    "created_at": "2026-02-24T21:48:21.015081Z"
  }
]
```

### `PATCH /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/channels/352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b`

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
  "id": "352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b",
  "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "name": "general-updated",
  "created_at": "2026-02-24T21:48:20.242852Z"
}
```

---

## VOICE CHANNELS

### `POST /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/voice`

Create a voice channel in a server. Only the server owner can do this. `max_users` is optional — omit for unlimited.

**Request Body:**

```json
{
  "name": "General Voice",
  "max_users": 10
}
```

**Response:**

```json
{
  "id": "6dabf9e5-de97-4b42-8518-f5146d954ea3",
  "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "name": "General Voice",
  "max_users": 10,
  "created_at": "2026-02-24T21:48:22.038714Z"
}
```

### `GET /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/voice`

List all voice channels in a server. Each entry includes a `participants` array with the current members and their mute/deafen state.

**Response:**

```json
[
  {
    "created_at": "2026-02-24T21:48:22.038714Z",
    "id": "6dabf9e5-de97-4b42-8518-f5146d954ea3",
    "max_users": 10,
    "name": "General Voice",
    "participants": [],
    "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d"
  },
  {
    "created_at": "2026-02-24T21:48:22.300170Z",
    "id": "1ce35600-f924-4544-b56d-f470e3622dbb",
    "max_users": null,
    "name": "Unlimited Voice",
    "participants": [],
    "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d"
  }
]
```

### `POST /voice/6dabf9e5-de97-4b42-8518-f5146d954ea3/join`

Join a voice channel. Inserts a participant row. Must be called before connecting the WebSocket stream. Handles reconnects idempotently.

**Response:**

```json
{
  "voice_channel_id": "6dabf9e5-de97-4b42-8518-f5146d954ea3",
  "user_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser",
  "avatar_url": "/avatars/e803e047-2e29-4c38-8643-182718caaa08.png",
  "muted": false,
  "deafened": false,
  "joined_at": "2026-02-24T21:48:23.338018Z"
}
```

### `GET /voice/6dabf9e5-de97-4b42-8518-f5146d954ea3/participants`

List all current participants in a voice channel with their mute/deafen state.

**Response:**

```json
[
  {
    "voice_channel_id": "6dabf9e5-de97-4b42-8518-f5146d954ea3",
    "user_id": "e803e047-2e29-4c38-8643-182718caaa08",
    "username": "testuser",
    "avatar_url": "/avatars/e803e047-2e29-4c38-8643-182718caaa08.png",
    "muted": false,
    "deafened": false,
    "joined_at": "2026-02-24T21:48:23.338018Z"
  },
  {
    "voice_channel_id": "6dabf9e5-de97-4b42-8518-f5146d954ea3",
    "user_id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
    "username": "otheruser",
    "avatar_url": null,
    "muted": false,
    "deafened": false,
    "joined_at": "2026-02-24T21:48:23.598829Z"
  }
]
```

### `PATCH /voice/6dabf9e5-de97-4b42-8518-f5146d954ea3/state`

Update the current user's mute or deafen state in a voice channel. Both fields are optional.

**Request Body:**

```json
{
  "muted": true
}
```

**Response:**

```json
{
  "voice_channel_id": "6dabf9e5-de97-4b42-8518-f5146d954ea3",
  "user_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser",
  "avatar_url": "/avatars/e803e047-2e29-4c38-8643-182718caaa08.png",
  "muted": true,
  "deafened": false,
  "joined_at": "2026-02-24T21:48:23.338018Z"
}
```

---

## MESSAGES

### `POST /channels/352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b/messages`

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
  "id": "4453b14e-4af6-4fe8-b1f9-14cbaf0e3226",
  "channel_id": "352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b",
  "user_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser",
  "content": "Hello world!",
  "reactions": [],
  "created_at": "2026-02-24T21:48:28.022245Z",
  "edited_at": null
}
```

### `GET /channels/352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b/messages`

Retrieve the last 50 messages in a channel, ordered oldest first. Includes the author's username and reactions.

**Response:**

```json
[
  {
    "id": "4453b14e-4af6-4fe8-b1f9-14cbaf0e3226",
    "channel_id": "352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b",
    "user_id": "e803e047-2e29-4c38-8643-182718caaa08",
    "username": "testuser",
    "content": "Hello world!",
    "reactions": [],
    "created_at": "2026-02-24T21:48:28.022245Z",
    "edited_at": null
  }
]
```

### `PATCH /channels/352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b/messages/4453b14e-4af6-4fe8-b1f9-14cbaf0e3226`

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
  "id": "4453b14e-4af6-4fe8-b1f9-14cbaf0e3226",
  "channel_id": "352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b",
  "user_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser",
  "content": "Edited!",
  "reactions": [],
  "created_at": "2026-02-24T21:48:28.022245Z",
  "edited_at": "2026-02-24T21:48:29.321874Z"
}
```

---

## REACTIONS

### `PUT /messages/4453b14e-4af6-4fe8-b1f9-14cbaf0e3226/reactions/👍`

Toggle a reaction ON for the current user. Idempotent — adding the same reaction twice has no effect. Returns the full updated message.

**Response:**

```json
{
  "id": "4453b14e-4af6-4fe8-b1f9-14cbaf0e3226",
  "channel_id": "352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b",
  "user_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser",
  "content": "Edited!",
  "reactions": [
    {
      "emoji_id": "\ud83d\udc4d",
      "count": 1,
      "user_ids": [
        "e803e047-2e29-4c38-8643-182718caaa08"
      ]
    }
  ],
  "created_at": "2026-02-24T21:48:28.022245Z",
  "edited_at": "2026-02-24T21:48:29.321874Z"
}
```

### `PUT /messages/4453b14e-4af6-4fe8-b1f9-14cbaf0e3226/reactions/🔥`

Multiple distinct emojis can be added to the same message.

**Response:**

```json
{
  "id": "4453b14e-4af6-4fe8-b1f9-14cbaf0e3226",
  "channel_id": "352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b",
  "user_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser",
  "content": "Edited!",
  "reactions": [
    {
      "emoji_id": "\ud83d\udc4d",
      "count": 2,
      "user_ids": [
        "e803e047-2e29-4c38-8643-182718caaa08",
        "58961f99-4163-47a3-a64d-df86ac11bd1c"
      ]
    },
    {
      "emoji_id": "\ud83d\udd25",
      "count": 1,
      "user_ids": [
        "e803e047-2e29-4c38-8643-182718caaa08"
      ]
    }
  ],
  "created_at": "2026-02-24T21:48:28.022245Z",
  "edited_at": "2026-02-24T21:48:29.321874Z"
}
```

### `DELETE /messages/4453b14e-4af6-4fe8-b1f9-14cbaf0e3226/reactions/🔥`

Remove a reaction for the current user. Returns 404 if the user hadn't added that reaction. Returns the full updated message.

**Response:**

```json
{
  "id": "4453b14e-4af6-4fe8-b1f9-14cbaf0e3226",
  "channel_id": "352c7fd9-9e7a-476b-a4f5-317f7a5f0f5b",
  "user_id": "e803e047-2e29-4c38-8643-182718caaa08",
  "username": "testuser",
  "content": "Edited!",
  "reactions": [
    {
      "emoji_id": "\ud83d\udc4d",
      "count": 2,
      "user_ids": [
        "e803e047-2e29-4c38-8643-182718caaa08",
        "58961f99-4163-47a3-a64d-df86ac11bd1c"
      ]
    }
  ],
  "created_at": "2026-02-24T21:48:28.022245Z",
  "edited_at": "2026-02-24T21:48:29.321874Z"
}
```

---

## ROLES

### `POST /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/roles`

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
  "id": "e1fd5724-2a3e-46aa-8f7d-2441fb60db98",
  "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "name": "Moderator",
  "permissions": 46,
  "created_at": "2026-02-24T21:48:33.221530Z"
}
```

### `GET /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/roles`

List all roles in a server ordered by creation time.

**Response:**

```json
[
  {
    "id": "e1fd5724-2a3e-46aa-8f7d-2441fb60db98",
    "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
    "name": "Moderator",
    "permissions": 46,
    "created_at": "2026-02-24T21:48:33.221530Z"
  },
  {
    "id": "19ec6234-22d6-499e-89a2-6db8e73e893b",
    "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
    "name": "Admin",
    "permissions": 128,
    "created_at": "2026-02-24T21:48:33.483068Z"
  }
]
```

---

## INVITES & MEMBERS

### `POST /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/invites`

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
  "id": "36b43895-e601-4ee6-aa55-e2224ded53cb",
  "code": "Q2gw4btQ",
  "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "created_by": "e803e047-2e29-4c38-8643-182718caaa08",
  "max_uses": 5,
  "uses": 0,
  "expires_at": null,
  "created_at": "2026-02-24T21:48:34.291128Z"
}
```

### `GET /invites/Q2gw4btQ`

Get invite metadata by code. Does not require authentication and does not consume a use.

**Response:**

```json
{
  "id": "36b43895-e601-4ee6-aa55-e2224ded53cb",
  "code": "Q2gw4btQ",
  "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "created_by": "e803e047-2e29-4c38-8643-182718caaa08",
  "max_uses": 5,
  "uses": 0,
  "expires_at": null,
  "created_at": "2026-02-24T21:48:34.291128Z"
}
```

### `POST /invites/Q2gw4btQ/join`

Use an invite code to join a server. Validates expiry and max uses. Returns the new `ServerMember` record.

**Response:**

```json
{
  "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "user_id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
  "joined_at": "2026-02-24T21:48:35.336178Z"
}
```

### `GET /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/members`

List all members of a server with their full profile data, ordered by join time.

**Response:**

```json
[
  {
    "id": "e803e047-2e29-4c38-8643-182718caaa08",
    "username": "testuser",
    "email": "test@test.com",
    "avatar_url": "/avatars/e803e047-2e29-4c38-8643-182718caaa08.png",
    "status": "dnd",
    "created_at": "2026-02-24T21:48:06.701994Z"
  },
  {
    "id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
    "username": "otheruser",
    "email": "other@test.com",
    "avatar_url": null,
    "status": "online",
    "created_at": "2026-02-24T21:48:07.315844Z"
  }
]
```

### `POST /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/roles/assign`

Assign a role to a server member. Requires `MANAGE_ROLES` permission. The user must already be a member.

**Request Body:**

```json
{
  "user_id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
  "role_id": "e1fd5724-2a3e-46aa-8f7d-2441fb60db98"
}
```

**Response:**

```json
{
  "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
  "user_id": "58961f99-4163-47a3-a64d-df86ac11bd1c",
  "role_id": "e1fd5724-2a3e-46aa-8f7d-2441fb60db98",
  "assigned_at": "2026-02-24T21:48:36.103626Z"
}
```

### `GET /servers/5f2ed5f7-fbe3-41a2-b425-b19b24598d9d/members/58961f99-4163-47a3-a64d-df86ac11bd1c/roles`

List all roles currently held by a specific user in a server.

**Response:**

```json
[
  {
    "id": "e1fd5724-2a3e-46aa-8f7d-2441fb60db98",
    "server_id": "5f2ed5f7-fbe3-41a2-b425-b19b24598d9d",
    "name": "Moderator",
    "permissions": 46,
    "created_at": "2026-02-24T21:48:33.221530Z"
  }
]
```

---

