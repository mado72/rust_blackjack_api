# Exemplos de Testes com cURL
# Blackjack API - Comandos prontos para copiar e colar

## Configuração
# Defina estas variáveis antes de executar os comandos
export BASE_URL="http://localhost:8080"
export GAME_ID=""
export JWT_TOKEN=""
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
## CRIAR JOGO (Público - sem autenticação)
## ============================================

# Criar jogo com 3 jogadores
# IMPORTANTE: Copie o game_id da resposta e salve na variável GAME_ID
curl -X POST "$BASE_URL/api/v1/games" \
  -H "Content-Type: application/json" \
  -d '{
    "emails": [
      "player1@example.com",
      "player2@example.com",
      "player3@example.com"
    ]
  }' \
  | jq '.'

# Salvar game_id automaticamente (Linux/Mac)
export GAME_ID=$(curl -s -X POST "$BASE_URL/api/v1/games" \
  -H "Content-Type: application/json" \
  -d '{"emails":["player1@example.com","player2@example.com"]}' \
  | jq -r '.game_id')
echo "Game ID: $GAME_ID"

## ============================================
## AUTENTICAÇÃO
## ============================================

# Login - Obter token JWT
# IMPORTANTE: Copie o token da resposta e salve na variável JWT_TOKEN
curl -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$PLAYER_EMAIL\",
    \"game_id\": \"$GAME_ID\"
  }" \
  | jq '.'

# Salvar token automaticamente (Linux/Mac)
export JWT_TOKEN=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$PLAYER_EMAIL\",\"game_id\":\"$GAME_ID\"}" \
  | jq -r '.token')
echo "Token: ${JWT_TOKEN:0:20}..."

## ============================================
## ESTADO DO JOGO (Requer autenticação)
## ============================================

# Ver estado atual do jogo
curl -X GET "$BASE_URL/api/v1/games/$GAME_ID" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '.'

## ============================================
## AÇÕES DO JOGADOR (Requer autenticação)
## ============================================

# Comprar uma carta
curl -X POST "$BASE_URL/api/v1/games/$GAME_ID/draw" \
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
      cartas_restantes: .cards_remaining
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

# FLUXO COMPLETO: Criar jogo, fazer login, jogar e finalizar
echo "=== Criando jogo ==="
GAME_ID=$(curl -s -X POST "$BASE_URL/api/v1/games" \
  -H "Content-Type: application/json" \
  -d '{"emails":["alice@example.com","bob@example.com"]}' \
  | jq -r '.game_id')
echo "Game ID: $GAME_ID"

echo -e "\n=== Fazendo login ==="
JWT_TOKEN=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"alice@example.com\",\"game_id\":\"$GAME_ID\"}" \
  | jq -r '.token')
echo "Token obtido"

echo -e "\n=== Comprando 3 cartas ==="
for i in {1..3}; do
  echo "Carta $i:"
  curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/draw" \
    -H "Authorization: Bearer $JWT_TOKEN" \
    -H "Content-Type: application/json" \
    | jq '{carta: .card.name, naipe: .card.suit, pontos: .current_points}'
done

echo -e "\n=== Finalizando jogo ==="
curl -s -X POST "$BASE_URL/api/v1/games/$GAME_ID/finish" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  | jq '{vencedor: .winner, pontos: .highest_score}'

## ============================================
## TESTES DE ERRO
## ============================================

# ERRO: Criar jogo sem jogadores (400 Bad Request)
curl -X POST "$BASE_URL/api/v1/games" \
  -H "Content-Type: application/json" \
  -d '{"emails":[]}' \
  -w "\nStatus: %{http_code}\n"

# ERRO: Criar jogo com muitos jogadores (400 Bad Request)
curl -X POST "$BASE_URL/api/v1/games" \
  -H "Content-Type: application/json" \
  -d '{
    "emails":[
      "p1@ex.com","p2@ex.com","p3@ex.com","p4@ex.com",
      "p5@ex.com","p6@ex.com","p7@ex.com","p8@ex.com",
      "p9@ex.com","p10@ex.com","p11@ex.com"
    ]
  }' \
  -w "\nStatus: %{http_code}\n"

# ERRO: Login com game_id inválido (400 Bad Request)
curl -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"player@example.com","game_id":"not-a-uuid"}' \
  -w "\nStatus: %{http_code}\n"

# ERRO: Login com jogador não existente (403 Forbidden)
curl -X POST "$BASE_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"hacker@example.com\",\"game_id\":\"$GAME_ID\"}" \
  -w "\nStatus: %{http_code}\n"

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
