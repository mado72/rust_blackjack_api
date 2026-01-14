# 📚 API Testing Resource Index

This document lists all available resources for testing the Blackjack API.

## ✅ Current Status (January 2026)

**Total Endpoints:** ✅ **16 HTTP endpoints fully functional**

All testing resources below work with the complete API. Turn-based gameplay, user management, and invitation system are all ready!

See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for complete feature details.

---

## 🎯 Where to Start?

### If you're new:
1. ✨ Start with **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Quick overview
2. 📖 Read **[POSTMAN_GUIDE.md](POSTMAN_GUIDE.md)** - Complete tutorial
3. 🚀 Use **Postman Collection** - The easiest way to test

### If you already know the API:
- 💻 Use **[api_tests.http](api_tests.http)** - quick tests in VS Code
- ⚡ Run **[test_api.ps1](test_api.ps1)** - Automated suite
- 📋 Consult **[CURL_EXAMPLES.md](CURL_EXAMPLES.md)** - Command reference

---

## 📁 Test Files

### For Postman

| File | Type | Description |
|---------|------|-----------|
| [Blackjack_API.postman_collection.json](Blackjack_API.postman_collection.json) | Collection | Complete collection with all endpoints |
| [Blackjack_API_Local.postman_environment.json](Blackjack_API_Local.postman_environment.json) | Environment | Variables for local environment |
| [POSTMAN_GUIDE.md](POSTMAN_GUIDE.md) | Documentation | Complete Postman usage guide |

**How to Use:**
1. Open Postman
2. Import → Select the 2 .json files
3. Select environment "Blackjack API - Local"
4. Follow guide in POSTMAN_GUIDE.md

**✅ Available Features:**
- Health checks (2 endpoints)
- User authentication (2 endpoints)
- Game management (4 endpoints)
- Gameplay actions (3 endpoints)
- Invitation system (5 endpoints)

---

### For VS Code

| File | Type | Description |
|---------|------|-----------|
| [api_tests.http](api_tests.http) | HTTP File | Ready-to-use requests for REST Client |

**How to Use:**
1. Install extension: `REST Client` (humao.rest-client)
2. Open file `api_tests.http`
3. Click "Send Request" above each request

**Features:**
- ✅ Configurable variables at top
- ✅ Examples for all endpoints
- ✅ Error tests included
- ✅ Explanatory comments

---

### Automated Scripts

| File | Language | Description |
|---------|-----------|-----------|
| [test_api.ps1](test_api.ps1) | PowerShell | Complete automated testing suite |

**How to Use:**
```powershell
# PowerShell Terminal
.\test_api.ps1
```

**What it does:**
- ✅ Tests all endpoints in sequence
- ✅ Validates responses
- ✅ Manages variables automatically
- ✅ Shows colored output
- ✅ Tests error scenarios
- ✅ Provides final summary

---

### Command Line

| Arquivo | Tipo | Descrição |
|---------|------|-----------|
| [CURL_EXAMPLES.md](CURL_EXAMPLES.md) | Documentation | Exemplos prontos com cURL |

**How to Use:**
- Copy and paste commands from file
- Adjust environment variables
- Works on Linux, Mac and Windows (Git Bash)

**Includes:**
- ✅ All endpoints
- ✅ Linux/Mac and Windows versions
- ✅ Examples with jq for formatting
- ✅ Complete test Scripts
- ✅ Tips and tricks

---

## 📖 Documentation

### Usage Guides

| Arquivo | Content | Target Audience |
|---------|----------|--------------|
| [POSTMAN_GUIDE.md](POSTMAN_GUIDE.md) | Tutorial completo do Postman | Beginners and intermediate |
| [QUICK_REFERENCE.md](QUICK_REFERENCE.md) | Referência rápida | All levels |
| [CURL_EXAMPLES.md](CURL_EXAMPLES.md) | Referência cURL | CLI developers |
| [README.md](../../README.md) | Project overview | Todos |

### Documentation Técnica

| Arquivo | Content |
|---------|----------|
| [docs/PRD.md](../PRD.md) | Product Requirements Document |
| [crates/blackjack-api/src/handlers.rs](../../crates/blackjack-api/src/handlers.rs) | Documentation inline dos endpoints |
| [crates/blackjack-api/config.toml](../../crates/blackjack-api/config.toml) | Default configuration |

---

## 🎓 Scenario-Based Tutorials

### 1. First Test (Postman)
**Time**: ~5 minutes

```
1. import collection in Postman
2. Health Check → send
3. Create Game → send (saves game_id)
4. Login → send (saves token)
5. Draw Card → send
6. Finish Game → send
```

**Arquivos necessários:**
- Blackjack_API.postman_collection.json
- Blackjack_API_Local.postman_environment.json

**Guia**: [POSTMAN_GUIDE.md](POSTMAN_GUIDE.md) - Section "Quick Test Flow"

---

### 2. Multi-Player Test
**Time**: ~10 minutes

```
1. Create game with 3 players
2. Login as player 1
3. Draw cards for player 1
4. Duplicate folder in Postman
5. Criar Variable player2_token
6. Login as player 2
7. Draw cards for player 2
8. Finish and see results
```

**Arquivos necessários:**
- Postman collection

**Guia**: [POSTMAN_GUIDE.md](POSTMAN_GUIDE.md) - Section "Testing with Multiple Players"

---

### 3. Complete Automated Test
**Time**: ~1 minute

```powershell
.\test_api.ps1
```

**Arquivos necessários:**
- test_api.ps1

**O que acontece:**
- Tests all endpoints
- Cria jogo, faz login, compra cartas
- Testa mudança de valor do Ás
- Finaliza e mostra resultados
- Tests error scenarios

---

### 4. Development with VS Code
**Time**: Continuous

```
1. Open api_tests.http in VS Code
2. Adjust variables at top
3. Click "Send Request" to test
4. Modify and re-test quickly
```

**Arquivos necessários:**
- api_tests.http
- REST Client extension

**Advantages:**
- ⚡ Very fast
- 📝 Easy to modify
- 💾 Versionable with git
- 🔄 Integrated in editor

---

### 5. CI/CD or Scripts
**Time**: Variable

Usar cURL para integração em pipelines:

```bash
# Ver CURL_EXAMPLES.md para exemplos completos
source CURL_EXAMPLES.md

# Example: basic test
./test_health_check.sh
```

**Arquivos necessários:**
- CURL_EXAMPLES.md (como referência)
- your own Scripts bash/PowerShell

---

## 🔧 Tools by Use Case

### Graphical Interface
**Use**: Postman  
**Quando**: Interactive testing, debugging, demonstrations  
**Arquivos**: `Blackjack_API.postman_collection.json`

### Code Editor
**Use**: VS Code REST Client  
**Quando**: Active development, quick tests  
**Arquivos**: `api_tests.http`

### Command Line
**Use**: cURL  
**Quando**: Scripts, CI/CD, automation  
**Arquivos**: `CURL_EXAMPLES.md`

### Automated Testing
**Use**: PowerShell Script  
**Quando**: Complete validation, regression  
**Arquivos**: `test_api.ps1`

---

## 📊 Feature Matrix

|  | Postman | VS Code | cURL | PowerShell |
|---|:---:|:---:|:---:|:---:|
| Graphical Interface | ✅ | ✅ | ❌ | ❌ |
| Auto-save variables | ✅ | ⚠️ | ❌ | ✅ |
| Documentation inline | ✅ | ✅ | ✅ | ✅ |
| Script testing | ✅ | ❌ | ❌ | ✅ |
| Versionable | ✅ | ✅ | ✅ | ✅ |
| Easy to share | ✅ | ✅ | ✅ | ✅ |
| CI/CD ready | ⚠️ | ⚠️ | ✅ | ✅ |
| Learning curve | Low | Low | Medium | Low |

**Legend:**
- ✅ Sim / Support completo
- ⚠️ Partial / With configuration
- ❌ No / Not recommended

---

## 🎯 Choose Your Tool

### You want...

**...test quickly during development?**
→ Use **VS Code REST Client** with `api_tests.http`

**...Documentation e compartilhamento?**
→ Use **Postman** with collections

**...automation e CI/CD?**
→ Use **cURL** or **PowerShell script**

**...learn the API for the first time?**
→ Start with **Postman** + **POSTMAN_GUIDE.md**

**...test everything at once?**
→ Run **test_api.ps1**

---

## 📞 Support

### Common Problems

**Variables not working**
- Postman: Check selected environment
- VS Code: Use syntax `@variavel = valor`
- cURL: Usar `export` no bash ou `$env:` no PowerShell

**Server not responding**
- Check if running: `cargo run -p blackjack-api`
- Check port: default `8080`
- See server logs for errors

**Token expired**
- Login again (`POST /api/v1/auth/login`)
- Token valid for 24 hours

### More Help

Consult:
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - Troubleshooting
- [POSTMAN_GUIDE.md](POSTMAN_GUIDE.md) - Section "Troubleshooting"
- [README.md](../../README.md) - Documentation principal

---

## 🚀 Next Steps

After testing the API:

1. **Integrate Frontend**: Use os endpoints para criar uma UI
2. **WebSocket**: Implementar notificações real-time (blueprint em websocket.rs)
3. **Persistence**: Adicionar SQLite (migrations já preparadas)
4. **Deploy**: Usar Dockerfile incluído

See [docs/PRD.md](../PRD.md) for complete roadmap.

---

**Maintained by**: Equipe Blackjack API  
**Last updated**: January 2026  
**API Version**: 1.0.0

