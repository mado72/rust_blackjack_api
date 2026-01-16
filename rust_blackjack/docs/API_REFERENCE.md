# API Reference Documentation for Rust Blackjack Backend

## Overview

This document provides a comprehensive reference for the API endpoints available in the Rust Blackjack backend. The API follows RESTful principles and is designed to facilitate interactions with the game services, including user authentication, game management, and invitations.

## Base URL

The base URL for all API requests is:

```
http://<your-server-address>/api/v1
```

## Authentication

### Login

- **Endpoint:** `POST /auth/login`
- **Description:** Authenticates a user and returns a JWT token.
- **Request Body:**
  ```json
  {
    "email": "user@example.com",
    "password": "yourpassword"
  }
  ```
- **Response:**
  - **200 OK**
    ```json
    {
      "token": "your_jwt_token"
    }
    ```
  - **401 Unauthorized**
    ```json
    {
      "message": "Invalid credentials"
    }
    ```

### Register

- **Endpoint:** `POST /auth/register`
- **Description:** Registers a new user.
- **Request Body:**
  ```json
  {
    "email": "user@example.com",
    "password": "yourpassword"
  }
  ```
- **Response:**
  - **201 Created**
    ```json
    {
      "message": "User registered successfully"
    }
    ```
  - **400 Bad Request**
    ```json
    {
      "message": "Email already exists"
    }
    ```

## Game Management

### Create Game

- **Endpoint:** `POST /games`
- **Description:** Creates a new game.
- **Request Body:**
  ```json
  {
    "creator_id": "uuid_of_creator",
    "enrollment_timeout_seconds": 300
  }
  ```
- **Response:**
  - **201 Created**
    ```json
    {
      "game_id": "uuid_of_game"
    }
    ```

### Get Game State

- **Endpoint:** `GET /games/:game_id`
- **Description:** Retrieves the current state of a game.
- **Response:**
  - **200 OK**
    ```json
    {
      "players": {
        "user@example.com": {
          "points": 20,
          "cards_history": [],
          "busted": false
        }
      },
      "cards_in_deck": 52,
      "finished": false
    }
    ```

### Draw Card

- **Endpoint:** `POST /games/:game_id/draw`
- **Description:** Allows a player to draw a card.
- **Request Body:**
  ```json
  {
    "email": "user@example.com"
  }
  ```
- **Response:**
  - **200 OK**
    ```json
    {
      "card": {
        "id": "uuid_of_card",
        "name": "Ace of Spades",
        "value": 11,
        "suit": "Spades"
      },
      "current_points": 21,
      "busted": false,
      "cards_remaining": 51,
      "cards_history": []
    }
    ```

### Finish Game

- **Endpoint:** `POST /games/:game_id/finish`
- **Description:** Finishes the game and calculates results.
- **Response:**
  - **200 OK**
    ```json
    {
      "winner": "user@example.com",
      "tied_players": [],
      "highest_score": 21,
      "all_players": {
        "user@example.com": {
          "points": 21,
          "cards_count": 5,
          "busted": false
        }
      }
    }
    ```

## Invitations

### Send Invitation

- **Endpoint:** `POST /games/:game_id/invitations`
- **Description:** Sends an invitation to a user to join a game.
- **Request Body:**
  ```json
  {
    "invitee_email": "invitee@example.com"
  }
  ```
- **Response:**
  - **201 Created**
    ```json
    {
      "message": "Invitation sent successfully"
    }
    ```

### Get Invitations

- **Endpoint:** `GET /games/:game_id/invitations`
- **Description:** Retrieves pending invitations for a game.
- **Response:**
  - **200 OK**
    ```json
    {
      "invitations": [
        {
          "id": "uuid_of_invitation",
          "invitee_email": "invitee@example.com",
          "status": "Pending"
        }
      ]
    }
    ```

## Error Handling

All API responses include a standardized error format:

- **Error Response:**
  ```json
  {
    "message": "Error description",
    "code": "ERROR_CODE",
    "status": 400,
    "details": {
      "field": "error detail"
    }
  }
  ```

## Conclusion

This API reference provides the necessary information to interact with the Rust Blackjack backend. For further details on specific endpoints or additional features, please refer to the relevant sections in the documentation.