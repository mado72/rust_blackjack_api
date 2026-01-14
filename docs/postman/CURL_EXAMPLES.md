# Exemplos de Testes com cURL
# Blackjack API - Comandos prontos para copiar e colar

## Configuração
# Defina estas variáveis antes de executar os comandos
export BASE_URL="http://localhost:8080"
export GAME_ID=""
export JWT_TOKEN=""
export USER_ID=""
export INVITATION_ID=""
export PLAYER_EMAIL="player1@example.com"
export CARD_ID=""

## ============================================
## HEALTH CHECKS (Público - sem autenticação)
## ============================================

# Health Check - Verifica status do servidor
curl -X GET "$BASE_URL/health" \
  -H "Content-Type: application/json" \
  | jq '.'

# Ready Check - Verifica prontidão dos componentes
curl -X GET "$BASE_URL/health/ready" \
  -H "Content-Type: application/json" \
  | jq '.'

## ============================================
## AUTENTICAÇÃO E USUÁRIOS (M7)
## ============================================

# Registrar novo usuário
curl -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "SecurePass123!"
  }' \
  | jq '.'

# Login de usuário (obter JWT token)
# IMPORTANTE: Copie o token da resposta e salve na variável JWT_TOKEN
curl -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "SecurePass123!"
  }' \
  | jq '.'

# Salvar token automaticamente (Linux/Mac/PowerShell)
export JWT_TOKEN=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"SecurePass123!"}' \
  | jq -r '.token')
echo "Token: ${JWT_TOKEN:0:20}..."

## ============================================
## GAME LIFECYCLE - ENROLLMENT SYSTEM (M7)
## ============================================

# Criar jogo com enrollment timeout (requer autenticação)
# IMPORTANTE: Copie o game_id da resposta
curl -X POST "$BASE_URL/api/v1/games" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "enrollment_timeout_seconds": 300
  }' \
  | jq '.'

# Salvar game_id automaticamente
export GAME_ID=$(curl -s -X POST "$BASE_URL/api/v1/games" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"enrollment_timeout_seconds":300}' \
  | jq -r '.game_id')
echo "Game ID: $GAME_ID"

# Listar jogos abertos para enrollment
curl -X GET "$BASE_URL/api/v1/games/open" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

# Enrollar jogador no jogo
curl -X POST "$BASE_URL/api/v1/games/$GAME_ID/enroll" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "bob@example.com"
  }' \
  | jq '.'

# Fechar enrollment (apenas criador pode fechar)
curl -X POST "$BASE_URL/api/v1/games/$GAME_ID/close-enrollment" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{}' \
  | jq '.'

## ============================================
## GAME INVITATIONS (M7 - PHASE 2A)
## ============================================

# Criar convite para outro jogador
curl -X POST "$BASE_URL/api/v1/games/$GAME_ID/invitations" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "invitee_email": "charlie@example.com"
  }' \
  | jq '.'

# Listar convites pendentes do usuário
curl -X GET "$BASE_URL/api/v1/invitations/pending" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

# Salvar invitation_id automaticamente
export INVITATION_ID=$(curl -s -X GET "$BASE_URL/api/v1/invitations/pending" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq -r '.invitations[0].invitation_id')
echo "Invitation ID: $INVITATION_ID"

# Aceitar convite (auto-enrolla no jogo)
curl -X POST "$BASE_URL/api/v1/invitations/$INVITATION_ID/accept" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

# Recusar convite
curl -X POST "$BASE_URL/api/v1/invitations/$INVITATION_ID/decline" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

## ============================================
## GAMEPLAY - TURN-BASED SYSTEM (M7)
## ============================================

# Ver estado atual do jogo (turn-based info)
curl -X GET "$BASE_URL/api/v1/games/$GAME_ID" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

# Comprar uma carta (valida se é seu turno)
curl -X POST "$BASE_URL/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

# Stand - Passar o turno (M7 - PHASE 2B)
curl -X POST "$BASE_URL/api/v1/games/$GAME_ID/stand" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

# Comprar carta e mostrar apenas informações relevantes
curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '{
      carta: .card.name,
      naipe: .card.suit,
      valor: .card.value,
      pontos_totais: .current_points,
      estourou: .busted,
      cartas_restantes: .cards_remaining,
      proximo_jogador: .next_player,
      jogo_finalizado: .is_finished
    }'

# Comprar carta e salvar card_id se for um Ás (Linux/Mac)
DRAW_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json")
echo "$DRAW_RESPONSE" | jq '.'
CARD_NAME=$(echo "$DRAW_RESPONSE" | jq -r '.card.name')
if [ "$CARD_NAME" = "A" ]; then
  export CARD_ID=$(echo "$DRAW_RESPONSE" | jq -r '.card.id')
  echo "Ás encontrado! Card ID: $CARD_ID"
fi

# Mudar valor do Ás para 1
curl -X PUT "$BASE_URL/api/v1/games/$GAME_ID/ace" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"card_id\": \"$CARD_ID\",
    \"as_eleven\": false
  }" \
  | jq '.'

# Mudar valor do Ás para 11
curl -X PUT "$BASE_URL/api/v1/games/$GAME_ID/ace" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"card_id\": \"$CARD_ID\",
    \"as_eleven\": true
  }" \
  | jq '.'

## ============================================
## FINALIZAR JOGO (Requer autenticação)
## ============================================

# Finalizar jogo e calcular vencedor
curl -X POST "$BASE_URL/api/v1/games/$GAME_ID/finish" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

# Ver resultados do jogo finalizado
curl -X GET "$BASE_URL/api/v1/games/$GAME_ID/results" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

# Ver resultados formatados
curl -s -X GET "$BASE_URL/api/v1/games/$GAME_ID/results" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '{
      vencedor: .winner,
      pontuacao_maxima: .highest_score,
      empatados: .tied_players,
      ranking: .all_players
    }'

## ============================================
## EXEMPLOS COMPLETOS
## ============================================

# FLUXO COMPLETO M7: Register → Login → Create → Enroll → Invite → Close → Play → Stand
echo "=== 1. Registrando usuários ==="
curl -s -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"Pass123!"}' \
  | jq '{user_id, email}'

curl -s -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"bob@example.com","password":"Pass123!"}' \
  | jq '{user_id, email}'

echo -e "\n=== 2. Login Alice ==="
ALICE_TOKEN=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"Pass123!"}' \
  | jq -r '.token')
echo "Alice token obtido"

echo -e "\n=== 3. Alice cria jogo ==="
GAME_ID=$(curl -s -X POST "$BASE_URL/api/v1/games" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"enrollment_timeout_seconds":300}' \
  | jq -r '.game_id')
echo "Game ID: $GAME_ID"

echo -e "\n=== 4. Login Bob ==="
BOB_TOKEN=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"bob@example.com","password":"Pass123!"}' \
  | jq -r '.token')
echo "Bob token obtido"

echo -e "\n=== 5. Bob se enrolla no jogo ==="
curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/enroll" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"email":"bob@example.com"}' \
  | jq '{message, enrolled_count}'

echo -e "\n=== 6. Alice fecha enrollment ==="
curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/close-enrollment" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{}' \
  | jq '{message, turn_order, player_count}'

echo -e "\n=== 7. Alice compra carta (primeiro turno) ==="
curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '{carta: .card.name, pontos: .current_points, proximo: .next_player}'

echo -e "\n=== 8. Alice dá stand ==="
curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/stand" \
  -H "Authorization: Bearer $ALICE_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '{message, is_finished, next_player}'

echo -e "\n=== 9. Bob compra carta ==="
curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '{carta: .card.name, pontos: .current_points}'

echo -e "\n=== 10. Bob dá stand (auto-finish) ==="
curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/stand" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '{message, is_finished, winner}'

## ============================================
## TESTES DE ERRO (M7)
## ============================================

# ERRO: Criar jogo sem autenticação (401 Unauthorized)
curl -X POST "$BASE_URL/api/v1/games" \
  -H "Content-Type: application/json" \
  -d '{"enrollment_timeout_seconds":300}' \
  -w "\nStatus: %{http_code}\n"

# ERRO: Registrar com email duplicado (409 Conflict)
curl -X POST "$BASE_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"Pass123!"}' \
  -w "\nStatus: %{http_code}\n"

# ERRO: Login com senha incorreta (401 Unauthorized)
curl -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"WrongPass"}' \
  -w "\nStatus: %{http_code}\n"

# ERRO: Comprar carta sem ser seu turno (409 NOT_YOUR_TURN)
curl -X POST "$BASE_URL/api/v1/games/$GAME_ID/draw" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -w "\nStatus: %{http_code}\n"

# ERRO: Fechar enrollment sem ser criador (403 Forbidden)
curl -X POST "$BASE_URL/api/v1/games/$GAME_ID/close-enrollment" \
  -H "Authorization: Bearer $BOB_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{}' \
  -w "\nStatus: %{http_code}\n"

# ERRO: Enrollar em jogo cheio (409 Game Full)
# (Precisa criar jogo com 10 players primeiro)

# ERRO: Aceitar convite expirado (410 Gone)
# (Precisa ter invitation_id expirado)

# ERRO: Acessar endpoint protegido sem token (401 Unauthorized)
curl -X GET "$BASE_URL/api/v1/games/$GAME_ID" \
  -H "Content-Type: application/json" \
  -w "\nStatus: %{http_code}\n"

# ERRO: Ver resultados de jogo não finalizado (409 Conflict)
curl -X GET "$BASE_URL/api/v1/games/$GAME_ID/results" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -w "\nStatus: %{http_code}\n"

## ============================================
## DICAS ÚTEIS
## ============================================

# Ver apenas o status HTTP da resposta
curl -X GET "$BASE_URL/health" -o /dev/null -w "%{http_code}\n" -s

# Ver headers da resposta
curl -X GET "$BASE_URL/health" -i

# Ver tempo de resposta
curl -X GET "$BASE_URL/health" \
  -w "\nTempo: %{time_total}s\n" \
  -o /dev/null -s

# Salvar resposta em arquivo
curl -X GET "$BASE_URL/api/v1/games/$GAME_ID" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -o game_state.json

# Fazer request em modo verbose (debug)
curl -v -X GET "$BASE_URL/health"

# Seguir redirects automaticamente
curl -L -X GET "$BASE_URL/health"

## ============================================
## VERSÃO WINDOWS (PowerShell)
## ============================================

# No PowerShell, use estas versões dos comandos:

# Criar jogo
$response = Invoke-RestMethod -Method POST -Uri "$env:BASE_URL/api/v1/games" `
  -ContentType "application/json" `
  -Body '{"emails":["player1@example.com","player2@example.com"]}'
$env:GAME_ID = $response.game_id

# Login
$response = Invoke-RestMethod -Method POST -Uri "$env:BASE_URL/api/v1/auth/login" `
  -ContentType "application/json" `
  -Body "{`"email`":`"$env:PLAYER_EMAIL`",`"game_id`":`"$env:GAME_ID`"}"
$env:JWT_TOKEN = $response.token

# Comprar carta
Invoke-RestMethod -Method POST -Uri "$env:BASE_URL/api/v1/games/$env:GAME_ID/draw" `
  -Headers @{"Authorization"="Bearer $env:JWT_TOKEN"} `
  -ContentType "application/json"

## ============================================
## NOTAS
## ============================================

# - Certifique-se de que o servidor está rodando em http://localhost:8080
# - Instale 'jq' para formatação JSON: https://stedolan.github.io/jq/
# - Use -s (silent) para suprimir barra de progresso do curl
# - Use -v (verbose) para debug detalhado
# - Tokens JWT expiram em 24 horas
# - Cada jogo tem seu próprio baralho de 52 cartas
