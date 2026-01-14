# 🧪 Blackjack API Testing Resources

This folder contains all necessary resources to test the Blackjack Multi-Player API.

## ✅ Status - January 2026 - COMPLETE!

**Turn-Based Multiplayer with User Management & Invitations**
- ✅ Complete backend infrastructure
- ✅ All 16 HTTP endpoints available
- ✅ Testable with Postman, cURL, and scripts

See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for complete feature list.

## 📦 Available Files

### Postman Collections
- **Blackjack_API.postman_collection.json** - Complete collection with all endpoints
- **Blackjack_API_Local.postman_environment.json** - Environment with pre-configured variables

### Usage Guides
- **[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md)** - 📊 Complete feature matrix and status
- **[POSTMAN_GUIDE.md](POSTMAN_GUIDE.md)** - Complete Postman tutorial (1,100+ lines)
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Quick reference guide (350+ lines)
- **[CURL_EXAMPLES.md](CURL_EXAMPLES.md)** - Ready-to-use cURL examples (450+ lines)
- **[API_TESTING_INDEX.md](API_TESTING_INDEX.md)** - Navigable index of all resources

### Testing Tools
- **api_tests.http** - File for VS Code REST Client extension
- **test_api.ps1** - PowerShell script for automated testing

## 🚀 Quick Start

### Option 1: Postman (Recommended)
1. Open Postman
2. Import → Select `Blackjack_API.postman_collection.json` and `Blackjack_API_Local.postman_environment.json`
3. Select environment "Blackjack API - Local"
4. Start with: **Health Check** → **Create Game** → **Login** → **Draw Card**

📖 [See complete guide](POSTMAN_GUIDE.md)

### Option 2: VS Code
1. Install extension **REST Client**
2. Open `api_tests.http`
3. Click "Send Request" above each endpoint

### Option 3: Automated Testing
```powershell
.\test_api.ps1
```

### Option 4: cURL
See [CURL_EXAMPLES.md](CURL_EXAMPLES.md) for ready-to-use examples.

## 📚 Documentation

### For Beginners
1. Start with [QUICK_REFERENCE.md](QUICK_REFERENCE.md) for overview
2. Read [POSTMAN_GUIDE.md](POSTMAN_GUIDE.md) for step-by-step tutorial
3. Use Postman collection for interactive testing

### For Experienced Developers
- Use [api_tests.http](api_tests.http) for quick tests
- Run [test_api.ps1](test_api.ps1) for complete suite
- Consult [CURL_EXAMPLES.md](CURL_EXAMPLES.md) for scripts

### Complete Navigation
See [API_TESTING_INDEX.md](API_TESTING_INDEX.md) for a complete index with scenario-based tutorials.

## ✨ Automatic Features

All tools include:
- ✅ Automatic JWT token management
- ✅ Automatic game_id saving
- ✅ Automatic card_id saving (for Aces)
- ✅ Complete inline documentation
- ✅ Error test examples
- ✅ Validation scripts

## 🔗 Useful Links

- [Main Documentation](../../README.md)
- [Product Requirements Document](../PRD.md)
- [API Source Code](../../crates/blackjack-api/)

## 📊 Tool Selection Matrix

| Situation | Recommended Tool | File |
|----------|------------------------|------|
| First test | Postman | `Blackjack_API.postman_collection.json` |
| Active development | VS Code REST Client | `api_tests.http` |
| Automated testing | PowerShell Script | `test_api.ps1` |
| CI/CD / Scripts | cURL | `CURL_EXAMPLES.md` |
| Learning the API | Postman Guide | `POSTMAN_GUIDE.md` |
| Quick reference | Quick Reference | `QUICK_REFERENCE.md` |

## 🆘 Need Help?

1. **Postman issues?** → [POSTMAN_GUIDE.md - Troubleshooting](POSTMAN_GUIDE.md#-troubleshooting)
2. **Common errors?** → [QUICK_REFERENCE.md - Common Errors](QUICK_REFERENCE.md#️-common-errors)
3. **Complete overview?** → [API_TESTING_INDEX.md](API_TESTING_INDEX.md)

## 📝 Note

Make sure the server is running before testing:
```bash
cargo run -p blackjack-api
# Server: http://localhost:8080
```

---

**Last updated**: January 2026  
**API Version**: 1.0.0
