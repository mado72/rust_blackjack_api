# Planejamento: Tornar Autenticação Obrigatória para Create Game

**Data:** 10 de Janeiro de 2026  
**Issue:** Tornar obrigatório usuário logado para criar jogos  
**Status Atual:** ❌ `create_game` aceita requisições sem autenticação  
**Status Alvo:** ✅ `create_game` requer autenticação JWT obrigatória  

## 📋 Análise do Estado Atual

### Endpoint `create_game`

**Localização:** [handlers.rs](../crates/blackjack-api/src/handlers.rs#L471-L494)

**Problema Identificado:**
```rust
#[tracing::instrument(skip(state))]
pub async fn create_game(
    State(state): State<crate::AppState>,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, ApiError> {
    // ❌ PROBLEMA: Não extrai Claims (JWT)
    // ❌ PROBLEMA: Usa UUID aleatório como creator_id
    let creator_id = Uuid::new_v4(); // Temporary placeholder
    let enrollment_timeout = payload.enrollment_timeout_seconds;
    let game_id = state.game_service.create_game(creator_id, enrollment_timeout)?;
    // ...
}
```

**Comentário no código atual:**
```rust
// TODO M7: Update to require authentication and use user_id as creator_id
```

### Rota Configurada

**Localização:** [main.rs](../crates/blackjack-api/src/main.rs#L127)

```rust
// M7: Game enrollment endpoints
.route("/api/v1/games", post(create_game))  // ❌ SEM middleware de auth
.route("/api/v1/games/open", get(get_open_games))
```

**Problema:** A rota está fora da camada protegida por autenticação JWT.

### Middleware de Autenticação

**Localização:** [main.rs](../crates/blackjack-api/src/main.rs#L135-L173)

```rust
// Protected routes requiring JWT authentication
.route("/api/v1/games/:game_id", get(get_game_state))
.route("/api/v1/games/:game_id/draw", post(draw_card))
// ... outras rotas protegidas
.layer(middleware::from_fn_with_state(
    app_state.clone(),
    jwt_middleware,
))
```

**Observação:** As rotas protegidas estão em um bloco separado com `.layer(jwt_middleware)`.

---

## 🎯 Objetivos

1. **Obrigatório:** Extrair `user_id` do JWT para usar como `creator_id`
2. **Obrigatório:** Rejeitar requisições sem autenticação (401 Unauthorized)
3. **Obrigatório:** Atualizar documentação do endpoint
4. **Obrigatório:** Atualizar testes para incluir JWT
5. **Obrigatório:** Atualizar coleção Postman
6. **Opcional:** Migrar dados existentes (se houver jogos com creator_id aleatório)

---

## 📝 Plano de Implementação

### FASE 1: Atualizar Handler `create_game`

**Arquivo:** `crates/blackjack-api/src/handlers.rs`

**Mudanças:**

1. **Adicionar extração de Claims:**
   ```rust
   #[tracing::instrument(skip(state, claims))]
   pub async fn create_game(
       State(state): State<crate::AppState>,
       Extension(claims): Extension<Claims>,  // ✅ NOVO
       Json(payload): Json<CreateGameRequest>,
   ) -> Result<Json<CreateGameResponse>, ApiError> {
   ```

2. **Usar user_id do JWT como creator_id:**
   ```rust
   // ✅ ANTES (placeholder aleatório):
   // let creator_id = Uuid::new_v4();
   
   // ✅ DEPOIS (ID real do usuário autenticado):
   let creator_id = claims.user_id;
   ```

3. **Atualizar logs:**
   ```rust
   tracing::info!(
       game_id = %game_id,
       creator_id = %creator_id,
       user_email = %claims.email,  // ✅ NOVO
       enrollment_timeout = ?enrollment_timeout,
       "Game created successfully by authenticated user"
   );
   ```

4. **Atualizar resposta (opcional):**
   ```rust
   Ok(Json(CreateGameResponse {
       game_id,
       creator_id,  // ✅ NOVO: Retornar creator_id
       message: "Game created successfully".to_string(),
       player_count: 1,  // ✅ ATUALIZADO: Creator é automaticamente jogador 1
   }))
   ```

5. **Atualizar estrutura de resposta:**
   ```rust
   #[derive(Debug, Serialize)]
   pub struct CreateGameResponse {
       pub game_id: Uuid,
       pub creator_id: Uuid,  // ✅ NOVO
       pub message: String,
       pub player_count: u32,
   }
   ```

**Complexidade:** 🟢 BAIXA  
**Tempo estimado:** 15 minutos  
**Risco:** 🟢 MÍNIMO (padrão já usado em outros endpoints)

---

### FASE 2: Mover Rota para Camada Autenticada

**Arquivo:** `crates/blackjack-api/src/main.rs`

**Mudanças:**

**Antes:**
```rust
// M7: Game enrollment endpoints
.route("/api/v1/games", post(create_game))  // ❌ Público
.route("/api/v1/games/open", get(get_open_games))
.route("/api/v1/games/:game_id/enroll", post(enroll_player))
.route(
    "/api/v1/games/:game_id/close-enrollment",
    post(close_enrollment),
)
// M7: User authentication endpoints
.route("/api/v1/auth/register", post(register_user))
.route("/api/v1/auth/login", post(login))
// ... rotas protegidas mais abaixo
```

**Depois:**
```rust
// M7: User authentication endpoints (públicos)
.route("/api/v1/auth/register", post(register_user))
.route("/api/v1/auth/login", post(login))
// M7: Game enrollment endpoints (protegidos)
.route("/api/v1/games", post(create_game))  // ✅ Movido para cima
.route("/api/v1/games/open", get(get_open_games))
.route("/api/v1/games/:game_id/enroll", post(enroll_player))
.route(
    "/api/v1/games/:game_id/close-enrollment",
    post(close_enrollment),
)
// M6: Game management (protegidos)
.route("/api/v1/games/:game_id", get(get_game_state))
.route("/api/v1/games/:game_id/draw", post(draw_card))
// ...
.layer(middleware::from_fn_with_state(
    app_state.clone(),
    jwt_middleware,
))  // ✅ Todos os endpoints acima agora protegidos
```

**Organização recomendada:**
```rust
let app = Router::new()
    // ============================================
    // PUBLIC ENDPOINTS (sem autenticação)
    // ============================================
    .route("/health", get(health_check))
    .route("/health/ready", get(ready_check))
    .route("/api/v1/auth/register", post(register_user))
    .route("/api/v1/auth/login", post(login))
    
    // ============================================
    // PROTECTED ENDPOINTS (requer JWT)
    // ============================================
    // M7: Game Enrollment
    .route("/api/v1/games", post(create_game))
    .route("/api/v1/games/open", get(get_open_games))
    .route("/api/v1/games/:game_id/enroll", post(enroll_player))
    .route("/api/v1/games/:game_id/close-enrollment", post(close_enrollment))
    
    // M6: Game Management
    .route("/api/v1/games/:game_id", get(get_game_state))
    .route("/api/v1/games/:game_id/draw", post(draw_card))
    .route("/api/v1/games/:game_id/ace", put(set_ace_value))
    .route("/api/v1/games/:game_id/stand", post(stand))
    .route("/api/v1/games/:game_id/finish", post(finish_game))
    .route("/api/v1/games/:game_id/results", get(get_game_results))
    
    // M6: Invitations
    .route("/api/v1/games/:game_id/invitations", post(create_invitation))
    .route("/api/v1/invitations/pending", get(get_pending_invitations))
    .route("/api/v1/invitations/:invitation_id/accept", post(accept_invitation))
    .route("/api/v1/invitations/:invitation_id/decline", post(decline_invitation))
    
    // Apply JWT authentication to all routes above
    .layer(middleware::from_fn_with_state(
        app_state.clone(),
        jwt_middleware,
    ))
    // ... rest of middleware stack
```

**Complexidade:** 🟢 BAIXA  
**Tempo estimado:** 10 minutos  
**Risco:** 🟡 MÉDIO (afeta ordem de rotas, pode quebrar endpoints se mal organizado)

---

### FASE 3: Atualizar Documentação do Endpoint

**Arquivo:** `crates/blackjack-api/src/handlers.rs`

**Mudanças na docstring de `create_game`:**

```rust
/// Creates a new game in enrollment mode
///
/// Initializes a new blackjack game in the Enrollment phase. The authenticated
/// user becomes the creator and first enrolled player. Other players can join
/// via the enrollment endpoint until the creator closes enrollment or the
/// timeout expires.
///
/// # Endpoint
///
/// `POST /api/v1/games`
///
/// # Authentication
///
/// **Required** - Must include valid JWT token in Authorization header.
/// The `user_id` from the JWT becomes the game creator.
///
/// # Request Body
///
/// ```json
/// {
///   "enrollment_timeout_seconds": 300  // Optional, default: 300 (5 min)
/// }
/// ```
///
/// # Response
///
/// **Success (200 OK)**:
/// ```json
/// {
///   "game_id": "550e8400-e29b-41d4-a716-446655440000",
///   "creator_id": "660e8400-e29b-41d4-a716-446655440000",
///   "message": "Game created successfully",
///   "player_count": 1
/// }
/// ```
///
/// # Errors
///
/// - **401 Unauthorized** - Missing or invalid JWT token
///   ```json
///   {
///     "message": "Authentication required",
///     "code": "UNAUTHORIZED",
///     "status": 401
///   }
///   ```
///
/// - **500 Internal Server Error** - Database or service error
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:8080/api/v1/games \
///   -H "Authorization: Bearer YOUR_JWT_TOKEN" \
///   -H "Content-Type: application/json" \
///   -d '{
///     "enrollment_timeout_seconds": 600
///   }'
/// ```
///
/// # Notes
///
/// - Creator is automatically enrolled as the first player
/// - Maximum 10 players per game
/// - Enrollment phase lasts `enrollment_timeout_seconds` (default: 300s)
/// - Creator can close enrollment early via `/close-enrollment` endpoint
```

**Complexidade:** 🟢 BAIXA  
**Tempo estimado:** 10 minutos  
**Risco:** 🟢 MÍNIMO

---

### FASE 4: Atualizar Testes

**Arquivo:** `crates/blackjack-api/tests/api_tests.rs`

**Mudanças:**

1. **Criar helper para obter JWT token:**
   ```rust
   async fn get_auth_token(client: &reqwest::Client, base_url: &str) -> String {
       let email = "testuser@example.com";
       let password = "TestPassword123!";
       
       // Register
       let _ = client
           .post(format!("{}/api/v1/auth/register", base_url))
           .json(&serde_json::json!({
               "email": email,
               "password": password
           }))
           .send()
           .await;
       
       // Login
       let response = client
           .post(format!("{}/api/v1/auth/login", base_url))
           .json(&serde_json::json!({
               "email": email,
               "password": password
           }))
           .send()
           .await
           .expect("Login failed");
       
       let login_response: serde_json::Value = response.json().await.unwrap();
       login_response["token"].as_str().unwrap().to_string()
   }
   ```

2. **Atualizar teste de `create_game`:**
   ```rust
   #[tokio::test]
   async fn test_create_game_requires_auth() {
       let base_url = setup_test_server().await;
       let client = reqwest::Client::new();
       
       // Test 1: Without auth - should fail
       let response = client
           .post(format!("{}/api/v1/games", base_url))
           .json(&serde_json::json!({
               "enrollment_timeout_seconds": 300
           }))
           .send()
           .await
           .unwrap();
       
       assert_eq!(response.status(), 401);
       
       // Test 2: With auth - should succeed
       let token = get_auth_token(&client, &base_url).await;
       
       let response = client
           .post(format!("{}/api/v1/games", base_url))
           .header("Authorization", format!("Bearer {}", token))
           .json(&serde_json::json!({
               "enrollment_timeout_seconds": 300
           }))
           .send()
           .await
           .unwrap();
       
       assert_eq!(response.status(), 200);
       
       let body: serde_json::Value = response.json().await.unwrap();
       assert!(body["game_id"].is_string());
       assert!(body["creator_id"].is_string());
       assert_eq!(body["player_count"], 1);
   }
   ```

3. **Atualizar outros testes que usam `create_game`:**
   - Adicionar helper `get_auth_token` em todos os testes
   - Adicionar header `Authorization` em todas as requisições para `/api/v1/games`

**Complexidade:** 🟡 MÉDIA  
**Tempo estimado:** 30 minutos  
**Risco:** 🟡 MÉDIO (pode quebrar testes existentes se não atualizar todos)

---

### FASE 5: Atualizar Coleção Postman

**Arquivo:** `docs/postman/Blackjack_API.postman_collection.json`

**Mudanças:**

1. **Atualizar request "Create Game (M7 - Enrollment Mode)":**
   ```json
   {
     "name": "Create Game (M7 - Enrollment Mode)",
     "request": {
       "auth": {
         "type": "bearer",
         "bearer": [
           {
             "key": "token",
             "value": "{{jwt_token}}",
             "type": "string"
           }
         ]
       },
       // ... rest of request
       "description": "**M7 FASE 1 - DISPONÍVEL ✅**\n\n**⚠️ AUTENTICAÇÃO OBRIGATÓRIA**\n\nCria um novo jogo no modo ENROLLMENT. O usuário autenticado torna-se o criador e primeiro jogador.\n\n..."
     }
   }
   ```

2. **Atualizar test script:**
   ```javascript
   // Salva o game_id e creator_id automaticamente
   if (pm.response.code === 200) {
       const response = pm.response.json();
       pm.collectionVariables.set('game_id', response.game_id);
       pm.collectionVariables.set('creator_id', response.creator_id);  // NOVO
       console.log('Game ID salvo:', response.game_id);
       console.log('Creator ID salvo:', response.creator_id);  // NOVO
       console.log('Status:', response.game_phase);
       console.log('Enrollment expira em:', response.enrollment_timeout);
   }
   ```

3. **Adicionar variável `creator_id`:**
   ```json
   {
     "key": "creator_id",
     "value": "",
     "type": "string"
   }
   ```

4. **Atualizar ordem de requests para garantir fluxo:**
   - Register User
   - Login (salva jwt_token)
   - Create Game (agora requer jwt_token)

**Complexidade:** 🟢 BAIXA  
**Tempo estimado:** 15 minutos  
**Risco:** 🟢 MÍNIMO

---

### FASE 6: Atualizar Documentação do Projeto

**Arquivos a atualizar:**

1. **`docs/PHASE1_COMPLETION.md`**
   - Seção "4.1 Create Game" - atualizar que agora requer autenticação
   - Adicionar nota sobre creator_id sendo extraído do JWT

2. **`docs/QUICK_REFERENCE.md`**
   - Atualizar exemplo curl para incluir header Authorization
   - Atualizar descrição de autenticação obrigatória

3. **`docs/postman/CURL_EXAMPLES.md`**
   - Atualizar exemplo de create game com Authorization header

4. **`docs/postman/POSTMAN_GUIDE.md`**
   - Adicionar passo obrigatório de Login antes de Create Game

5. **`docs/PRD.md`**
   - Atualizar seção M7 para refletir que create_game agora requer auth
   - Marcar TODO como concluído

**Complexidade:** 🟢 BAIXA  
**Tempo estimado:** 20 minutos  
**Risco:** 🟢 MÍNIMO

---

## 🔧 Mudanças Adicionais (Opcional)

### OPCIONAL 1: Migrar Dados Existentes

Se houver jogos no banco de dados com `creator_id` aleatório (UUID v4), considerar:

```rust
// Script de migração (se necessário)
async fn migrate_creator_ids() {
    // 1. Identificar jogos com creator_id não associado a user
    // 2. Atualizar para usar user_id do primeiro jogador
    // OU
    // 3. Deletar jogos de teste sem creator válido
}
```

**Complexidade:** 🔴 ALTA  
**Tempo estimado:** 1-2 horas  
**Risco:** 🔴 ALTO (pode perder dados)  
**Recomendação:** Apenas se houver dados de produção

---

### OPCIONAL 2: Adicionar Validação de Creator em Service Layer

**Arquivo:** `crates/blackjack-service/src/lib.rs`

Adicionar validação que `creator_id` existe na tabela `users`:

```rust
pub fn create_game(
    &self,
    creator_id: Uuid,
    enrollment_timeout_seconds: Option<u64>,
) -> Result<Uuid, GameServiceError> {
    // ✅ NOVO: Validar que creator_id existe
    let user_exists = self.db.query(
        "SELECT id FROM users WHERE id = $1",
        &[&creator_id],
    )?;
    
    if user_exists.is_empty() {
        return Err(GameServiceError::InvalidCreator(
            "Creator user does not exist".to_string()
        ));
    }
    
    // ... resto da lógica
}
```

**Complexidade:** 🟡 MÉDIA  
**Tempo estimado:** 30 minutos  
**Risco:** 🟡 MÉDIO  
**Benefício:** Garante integridade referencial

---

## ✅ Checklist de Implementação

### Código
- [ ] **FASE 1:** Atualizar handler `create_game` para extrair `Extension(claims)`
- [ ] **FASE 1:** Usar `claims.user_id` como `creator_id`
- [ ] **FASE 1:** Atualizar `CreateGameResponse` para incluir `creator_id`
- [ ] **FASE 2:** Mover rota `/api/v1/games` para camada autenticada em `main.rs`
- [ ] **FASE 2:** Reorganizar rotas (public vs protected)
- [ ] **FASE 3:** Atualizar docstring de `create_game`

### Testes
- [ ] **FASE 4:** Criar helper `get_auth_token()`
- [ ] **FASE 4:** Adicionar teste `test_create_game_requires_auth()`
- [ ] **FASE 4:** Atualizar testes existentes para incluir JWT
- [ ] **FASE 4:** Executar `cargo test` e validar 100% passing

### Documentação
- [ ] **FASE 5:** Atualizar Postman collection (auth + test script)
- [ ] **FASE 5:** Adicionar variável `creator_id` no Postman
- [ ] **FASE 6:** Atualizar `PHASE1_COMPLETION.md`
- [ ] **FASE 6:** Atualizar `QUICK_REFERENCE.md`
- [ ] **FASE 6:** Atualizar `CURL_EXAMPLES.md`
- [ ] **FASE 6:** Atualizar `POSTMAN_GUIDE.md`
- [ ] **FASE 6:** Atualizar `PRD.md`

### Validação
- [ ] Compilar: `cargo build --release`
- [ ] Testes: `cargo test` (78+ tests passing)
- [ ] Lint: `cargo clippy -- -D warnings`
- [ ] Format: `cargo fmt --check`
- [ ] Executar servidor: `cargo run -p blackjack-api`
- [ ] Testar manualmente com Postman:
  - [ ] Create game sem auth → 401
  - [ ] Create game com auth → 200 + creator_id correto
  - [ ] Verificar que creator_id == user_id do JWT

### Git
- [ ] Commit mudanças de código
- [ ] Commit mudanças de testes
- [ ] Commit mudanças de documentação
- [ ] Commit mudanças do Postman
- [ ] Push para branch `feature/M7`

---

## 📊 Resumo de Impacto

| Aspecto | Antes | Depois |
|---------|-------|--------|
| **Autenticação** | ❌ Opcional | ✅ Obrigatória |
| **Creator ID** | 🔄 UUID aleatório | ✅ user_id do JWT |
| **Status Code sem auth** | 200 OK | 401 Unauthorized |
| **Segurança** | 🔴 Baixa | 🟢 Alta |
| **Integridade de dados** | 🟡 Média | 🟢 Alta |
| **Postman auth** | `"type": "noauth"` | `"type": "bearer"` |
| **Player count inicial** | 0 | 1 (creator) |

---

## ⚡ Ordem de Execução Recomendada

```
1. FASE 1 (15 min) → Atualizar handler
2. FASE 2 (10 min) → Mover rota
3. Compilar e testar manualmente (5 min)
4. FASE 3 (10 min) → Atualizar docs do código
5. FASE 4 (30 min) → Atualizar testes
6. Executar cargo test (5 min)
7. FASE 5 (15 min) → Atualizar Postman
8. FASE 6 (20 min) → Atualizar docs gerais
9. Validação final (10 min)
10. Git commits (5 min)

TOTAL ESTIMADO: ~2 horas
```

---

## 🚨 Riscos e Mitigações

### Risco 1: Quebrar Backward Compatibility
**Probabilidade:** 🔴 ALTA  
**Impacto:** 🔴 ALTO  
**Mitigação:**
- Esta é uma **BREAKING CHANGE** intencional
- Documentar claramente em CHANGELOG
- Se houver clientes usando a API, notificar com antecedência
- Considerar versioning da API (v1 vs v2)

### Risco 2: Testes Falhando
**Probabilidade:** 🟡 MÉDIA  
**Impacto:** 🟡 MÉDIO  
**Mitigação:**
- Atualizar TODOS os testes que chamam `create_game`
- Usar helper `get_auth_token` consistentemente
- Executar `cargo test` após cada mudança incremental

### Risco 3: Dados Órfãos no Banco
**Probabilidade:** 🟢 BAIXA (ambiente dev)  
**Impacto:** 🟡 MÉDIO  
**Mitigação:**
- Em desenvolvimento: limpar banco de dados de teste
- Em produção: script de migração (OPCIONAL 1)

---

## 📚 Referências

- **JWT Authentication Pattern:** Outros endpoints já implementados (get_open_games, enroll_player)
- **Middleware Stack:** [main.rs](../crates/blackjack-api/src/main.rs#L135-L173)
- **Claims Structure:** [auth.rs](../crates/blackjack-api/src/auth.rs)
- **Documentação M7 FASE 1:** [PHASE1_COMPLETION.md](./PHASE1_COMPLETION.md)

---

## ✨ Benefícios Esperados

1. **Segurança:** Apenas usuários autenticados podem criar jogos
2. **Rastreabilidade:** Cada jogo tem um criador identificável
3. **Integridade:** creator_id é garantidamente um user_id válido
4. **Consistência:** Padrão de autenticação uniforme em toda a API
5. **Auditoria:** Logs incluem email do criador
6. **Features futuras:** Facilita implementar "meus jogos", estatísticas por usuário, etc.

---

**Prioridade:** 🔴 ALTA  
**Categoria:** Security & Data Integrity  
**Milestone:** M7 FASE 1 (Complementar)  
**Estimativa total:** 2 horas  
