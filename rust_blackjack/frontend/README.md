# Frontend React Project for Rust Blackjack API

This README provides an overview of the frontend React project that consumes the API services of the Rust Blackjack backend. 

## Project Structure

The frontend project is organized as follows:

```
frontend
├── public
│   ├── index.html        # Main HTML file for the React application
│   └── favicon.ico       # Favicon for the web application
├── src
│   ├── api               # API service files for authentication, games, and invitations
│   ├── components        # React components for authentication, game, lobby, and shared UI
│   ├── contexts          # Contexts for managing global state (e.g., authentication)
│   ├── hooks             # Custom hooks for managing logic related to authentication and games
│   ├── pages             # Page components for routing
│   ├── types             # TypeScript types for API responses and data structures
│   ├── utils             # Utility functions for validation and formatting
│   ├── App.tsx           # Main application component
│   ├── index.tsx         # Entry point for the React application
│   └── index.css         # Global styles for the application
├── package.json          # npm configuration file
└── tsconfig.json         # TypeScript configuration file
```

## Getting Started

To get started with the frontend project, follow these steps:

1. **Clone the Repository**
   ```bash
   git clone https://github.com/mado72/rust_blackjack_api.git
   cd rust_blackjack_api/frontend
   ```

2. **Install Dependencies**
   Make sure you have Node.js and npm installed. Then run:
   ```bash
   npm install
   ```

3. **Run the Development Server**
   Start the development server with:
   ```bash
   npm start
   ```
   This will launch the application in your default web browser at `http://localhost:3000`.

## API Integration

The frontend interacts with the Rust Blackjack backend through the following API endpoints:

- **Authentication**
  - Login and registration functionalities are handled in `src/api/auth.ts`.

- **Game Management**
  - Game creation and management are implemented in `src/api/games.ts`.

- **Invitations**
  - Game invitations are managed through `src/api/invitations.ts`.

## Components Overview

The application is structured into several components:

- **Auth Components**
  - `LoginForm`: Handles user login.
  - `RegisterForm`: Handles user registration.

- **Game Components**
  - `GameBoard`: Displays the game board.
  - `PlayerHand`: Shows the player's hand of cards.
  - `DealerHand`: Displays the dealer's hand.
  - `GameControls`: Provides controls for gameplay actions.

- **Lobby Components**
  - `GameLobby`: Displays available games for players to join.
  - `OpenGames`: Lists open games.
  - `Invitations`: Shows invitations for players.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any enhancements or bug fixes.

## License

This project is licensed under the MIT License. See the LICENSE file for details.