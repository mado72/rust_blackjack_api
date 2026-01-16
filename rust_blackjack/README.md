# Rust Blackjack Project

This project is a multi-player blackjack game implemented with a Rust backend and a React frontend. The backend provides a RESTful API for managing game logic, user authentication, and real-time interactions, while the frontend offers a user-friendly interface for players to engage with the game.

## Project Structure

The project is organized into several directories:

- **crates/**: Contains the Rust backend services.
  - **blackjack-api/**: The API service that handles HTTP requests and responses.
  - **blackjack-cli/**: The command-line interface for the blackjack game.
  - **blackjack-core/**: The core game logic and data structures.
  - **blackjack-service/**: The service layer that manages game state and interactions.

- **frontend/**: Contains the React frontend application.
  - **public/**: Static files served by the frontend.
    - **index.html**: The main HTML file for the React application.
    - **favicon.ico**: The favicon for the web application.
  - **src/**: Source code for the React application.
    - **api/**: Functions for interacting with the backend API.
    - **components/**: React components for different parts of the application.
    - **contexts/**: Context providers for managing global state.
    - **hooks/**: Custom hooks for reusable logic.
    - **pages/**: Page components for routing.
    - **types/**: TypeScript types for API responses and data structures.
    - **utils/**: Utility functions for validation and formatting.
    - **App.tsx**: The main application component.
    - **index.tsx**: The entry point for the React application.
    - **index.css**: Global styles for the application.

- **docs/**: Documentation for the project.
  - **PRD.md**: Product requirements document.
  - **SECURITY.md**: Security-related documentation.
  - **API_REFERENCE.md**: API reference documentation.

- **Cargo.toml**: Configuration file for the Rust project.

## Getting Started

### Prerequisites

- Rust and Cargo installed for the backend.
- Node.js and npm installed for the frontend.

### Backend Setup

1. Navigate to the `crates/blackjack-api` directory.
2. Run `cargo build` to build the backend services.
3. Run `cargo run` to start the backend server.

### Frontend Setup

1. Navigate to the `frontend` directory.
2. Run `npm install` to install the frontend dependencies.
3. Run `npm start` to start the React application.

### API Documentation

Refer to the `docs/API_REFERENCE.md` for detailed information on the available API endpoints and their usage.

## Contributing

Contributions are welcome! Please submit a pull request or open an issue for any enhancements or bug fixes.

## License

This project is licensed under the MIT License. See the LICENSE file for more details.