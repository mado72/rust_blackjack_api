# MCP Server Configuration Guide

## Overview
This project uses Model Context Protocol (MCP) servers to enhance development workflow for the Rust Blackjack REST API backend.

## Configured Servers

### 1. Filesystem Server
**Purpose**: Manage workspace structure with multiple crates
- Navigate between 4 crates (core, service, api, cli)
- Edit configuration files (config.toml, .env)
- Manage SQL migrations
- Update documentation

### 2. GitHub Server
**Purpose**: CI/CD and repository management
- Manage GitHub Actions workflows
- Create and review pull requests
- Track issues for each phase
- Integrate automated testing
- Always use Git Docs Pattern's in git's comment (chore, feat, docs, test, build, refact, perform...)

**Setup Required**:
1. Create GitHub Personal Access Token at: https://github.com/settings/tokens
2. Grant permissions: `repo`, `workflow`, `read:org`
3. Replace `<your-token-here>` in `mcp-settings.json` with your token
4. **IMPORTANT**: Never commit your token to version control

### 3. Memory Server
**Purpose**: Maintain context across development phases
- Remember design decisions
- Track phase completion status
- Maintain acceptance criteria checklist

## Installation

### Prerequisites
- Node.js and npm installed
- npx available in PATH

### Verify Installation
```powershell
npx -y @modelcontextprotocol/server-filesystem --version
npx -y @modelcontextprotocol/server-github --version
npx -y @modelcontextprotocol/server-memory --version
```

## Usage by Phase

### Phase 1: Workspace Configuration
- **Primary**: Filesystem
- Create workspace Cargo.toml
- Setup crate structure
- Create GitHub workflow

### Phase 2-3: Core and Service Implementation
- **Primary**: Filesystem + Memory
- Implement game logic
- Track design patterns
- Manage complex state

### Phase 4-5: API and REST Endpoints
- **Primary**: Filesystem + GitHub
- Configure external settings
- Setup CI/CD pipeline
- Document API endpoints

### Phase 6: Testing and Deployment
- **Primary**: All three servers
- Run integration tests
- Build Docker images
- Deploy via GitHub Actions

## Security Notes

1. **GitHub Token**: Store in environment variable or secure vault
2. **Never commit**: Add `.vscode/mcp-settings.json` to `.gitignore` if it contains secrets
3. **Token rotation**: Regularly rotate GitHub tokens
4. **Minimal permissions**: Only grant necessary token scopes

## Future Enhancements

When implementing database persistence (Phase 6+):
```json
"postgresql": {
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-postgresql"],
  "env": {
    "POSTGRES_CONNECTION_STRING": "postgresql://user:pass@localhost/blackjack"
  }
}
```

## Troubleshooting

### Server not found
```powershell
npm cache clean --force
npx clear-npx-cache
```

### Permission denied
Run PowerShell as Administrator or check execution policy:
```powershell
Get-ExecutionPolicy
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### GitHub authentication fails
- Verify token has correct scopes
- Check token hasn't expired
- Ensure token is properly set in environment

## References

- [MCP Documentation](https://modelcontextprotocol.io)
- [Filesystem Server](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem)
- [GitHub Server](https://github.com/modelcontextprotocol/servers/tree/main/src/github)
- [Memory Server](https://github.com/modelcontextprotocol/servers/tree/main/src/memory)
