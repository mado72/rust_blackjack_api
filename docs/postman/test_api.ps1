# Script de Teste Automatizado - Blackjack API
# Este script testa todos os endpoints em uma sequência lógica

$ErrorActionPreference = "Continue"
$baseUrl = "http://localhost:8080"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "Blackjack API - Testes Automatizados" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Função para fazer requests e mostrar resultados
function Invoke-ApiTest {
    param(
        [string]$Method,
        [string]$Uri,
        [string]$Body = $null,
        [hashtable]$Headers = @{},
        [string]$Description
    )
    
    Write-Host "► $Description" -ForegroundColor Yellow
    Write-Host "  $Method $Uri" -ForegroundColor Gray
    
    try {
        $params = @{
            Method = $Method
            Uri = $Uri
            Headers = $Headers
            ContentType = "application/json"
        }
        
        if ($Body) {
            $params.Body = $Body
        }
        
        $response = Invoke-RestMethod @params
        Write-Host "  ✓ Sucesso" -ForegroundColor Green
        return $response
    }
    catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        Write-Host "  ✗ Erro: $statusCode" -ForegroundColor Red
        if ($_.ErrorDetails.Message) {
            $err = $_.ErrorDetails.Message | ConvertFrom-Json
            Write-Host "    $($err.message)" -ForegroundColor Red
        }
        return $null
    }
    finally {
        Write-Host ""
    }
}

# 1. Health Checks
Write-Host "=== HEALTH CHECKS ===" -ForegroundColor Magenta
$health = Invoke-ApiTest -Method GET -Uri "$baseUrl/health" -Description "Health Check"
if ($health) {
    Write-Host "  Status: $($health.status)" -ForegroundColor Cyan
    Write-Host "  Uptime: $($health.uptime_seconds)s" -ForegroundColor Cyan
    Write-Host "  Version: $($health.version)" -ForegroundColor Cyan
    Write-Host ""
}

$ready = Invoke-ApiTest -Method GET -Uri "$baseUrl/health/ready" -Description "Ready Check"
if ($ready) {
    Write-Host "  Ready: $($ready.ready)" -ForegroundColor Cyan
    Write-Host "  Checks: $($ready.checks | ConvertTo-Json -Compress)" -ForegroundColor Cyan
    Write-Host ""
}

# 2. Create Game
Write-Host "=== CRIAR JOGO ===" -ForegroundColor Magenta
$createGameBody = @{
    emails = @(
        "player1@example.com",
        "player2@example.com",
        "player3@example.com"
    )
} | ConvertTo-Json

$game = Invoke-ApiTest -Method POST -Uri "$baseUrl/api/v1/games" -Body $createGameBody -Description "Criar jogo com 3 jogadores"
if ($game) {
    $gameId = $game.game_id
    Write-Host "  Game ID: $gameId" -ForegroundColor Cyan
    Write-Host "  Jogadores: $($game.player_count)" -ForegroundColor Cyan
    Write-Host "  Mensagem: $($game.message)" -ForegroundColor Cyan
    Write-Host ""
}
else {
    Write-Host "Não foi possível criar o jogo. Abortando testes." -ForegroundColor Red
    exit 1
}

# 3. Login Player 1
Write-Host "=== AUTENTICAÇÃO ===" -ForegroundColor Magenta
$loginBody = @{
    email = "player1@example.com"
    game_id = $gameId
} | ConvertTo-Json

$auth = Invoke-ApiTest -Method POST -Uri "$baseUrl/api/v1/auth/login" -Body $loginBody -Description "Login como player1@example.com"
if ($auth) {
    $token = $auth.token
    $headers = @{
        "Authorization" = "Bearer $token"
    }
    Write-Host "  Token recebido (primeiros 20 chars): $($token.Substring(0, 20))..." -ForegroundColor Cyan
    Write-Host "  Expira em: $($auth.expires_in)s" -ForegroundColor Cyan
    Write-Host ""
}
else {
    Write-Host "Não foi possível fazer login. Abortando testes." -ForegroundColor Red
    exit 1
}

# 4. Get Game State (antes de jogar)
Write-Host "=== ESTADO DO JOGO ===" -ForegroundColor Magenta
$state = Invoke-ApiTest -Method GET -Uri "$baseUrl/api/v1/games/$gameId" -Headers $headers -Description "Ver estado inicial do jogo"
if ($state) {
    Write-Host "  Cartas no baralho: $($state.cards_in_deck)" -ForegroundColor Cyan
    Write-Host "  Finalizado: $($state.finished)" -ForegroundColor Cyan
    Write-Host "  Jogadores: $($state.players.Count)" -ForegroundColor Cyan
    foreach ($player in $state.players.PSObject.Properties) {
        Write-Host "    - $($player.Name): $($player.Value.points) pontos, $($player.Value.cards_history.Count) cartas" -ForegroundColor Gray
    }
    Write-Host ""
}

# 5. Draw Cards
Write-Host "=== COMPRAR CARTAS ===" -ForegroundColor Magenta
$aceCardId = $null

for ($i = 1; $i -le 3; $i++) {
    $draw = Invoke-ApiTest -Method POST -Uri "$baseUrl/api/v1/games/$gameId/draw" -Headers $headers -Description "Comprar carta #$i"
    if ($draw) {
        Write-Host "  Carta: $($draw.card.name) de $($draw.card.suit) (valor: $($draw.card.value))" -ForegroundColor Cyan
        Write-Host "  Pontos totais: $($draw.current_points)" -ForegroundColor Cyan
        Write-Host "  Estourou: $($draw.busted)" -ForegroundColor Cyan
        Write-Host "  Cartas restantes: $($draw.cards_remaining)" -ForegroundColor Cyan
        
        # Salvar ID de um Ás se encontrar
        if ($draw.card.name -eq "Ace" -and $null -eq $aceCardId) {
            $aceCardId = $draw.card.id
            Write-Host "  ► Ás encontrado! ID salvo para teste." -ForegroundColor Yellow
        }
        Write-Host ""
    }
}

# 6. Set Ace Value (se tiver um Ás)
if ($aceCardId) {
    Write-Host "=== MUDAR VALOR DO ÁS ===" -ForegroundColor Magenta
    
    # Mudar para 1
    $aceBody = @{
        card_id = $aceCardId
        as_eleven = $false
    } | ConvertTo-Json
    
    $ace = Invoke-ApiTest -Method PUT -Uri "$baseUrl/api/v1/games/$gameId/ace" -Headers $headers -Body $aceBody -Description "Mudar Ás para valor 1"
    if ($ace) {
        Write-Host "  Novos pontos: $($ace.points)" -ForegroundColor Cyan
        Write-Host "  Estourou: $($ace.busted)" -ForegroundColor Cyan
        Write-Host ""
    }
    
    # Mudar de volta para 11
    $aceBody = @{
        card_id = $aceCardId
        as_eleven = $true
    } | ConvertTo-Json
    
    $ace = Invoke-ApiTest -Method PUT -Uri "$baseUrl/api/v1/games/$gameId/ace" -Headers $headers -Body $aceBody -Description "Mudar Ás para valor 11"
    if ($ace) {
        Write-Host "  Novos pontos: $($ace.points)" -ForegroundColor Cyan
        Write-Host "  Estourou: $($ace.busted)" -ForegroundColor Cyan
        Write-Host ""
    }
}

# 7. Get Game State (depois de jogar)
Write-Host "=== ESTADO ATUALIZADO ===" -ForegroundColor Magenta
$state = Invoke-ApiTest -Method GET -Uri "$baseUrl/api/v1/games/$gameId" -Headers $headers -Description "Ver estado após comprar cartas"
if ($state) {
    Write-Host "  Cartas no baralho: $($state.cards_in_deck)" -ForegroundColor Cyan
    $player1 = $state.players.'player1@example.com'
    Write-Host "  Player 1 pontos: $($player1.points)" -ForegroundColor Cyan
    Write-Host "  Player 1 cartas: $($player1.cards_history.Count)" -ForegroundColor Cyan
    Write-Host ""
}

# 8. Finish Game
Write-Host "=== FINALIZAR JOGO ===" -ForegroundColor Magenta
$finish = Invoke-ApiTest -Method POST -Uri "$baseUrl/api/v1/games/$gameId/finish" -Headers $headers -Description "Finalizar jogo e calcular vencedor"
if ($finish) {
    Write-Host "  Vencedor: $($finish.winner)" -ForegroundColor Green
    Write-Host "  Pontuação mais alta: $($finish.highest_score)" -ForegroundColor Cyan
    Write-Host "  Jogadores empatados: $($finish.tied_players.Count)" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Ranking completo:" -ForegroundColor Yellow
    foreach ($player in $finish.all_players.PSObject.Properties) {
        $status = if ($player.Value.busted) { "BUST" } else { "OK" }
        Write-Host "    $($player.Name): $($player.Value.points) pontos ($($player.Value.cards_count) cartas) [$status]" -ForegroundColor Gray
    }
    Write-Host ""
}

# 9. Get Results
Write-Host "=== RESULTADOS FINAIS ===" -ForegroundColor Magenta
$results = Invoke-ApiTest -Method GET -Uri "$baseUrl/api/v1/games/$gameId/results" -Headers $headers -Description "Ver resultados do jogo"
if ($results) {
    Write-Host "  Vencedor: $($results.winner)" -ForegroundColor Green
    Write-Host "  Pontuação máxima: $($results.highest_score)" -ForegroundColor Cyan
    Write-Host ""
}

# 10. Testes de Erro
Write-Host "=== TESTES DE ERRO ===" -ForegroundColor Magenta

# Tentar comprar carta de jogo finalizado
Write-Host "► Tentando comprar carta de jogo finalizado (deve falhar)" -ForegroundColor Yellow
try {
    $null = Invoke-RestMethod -Method POST -Uri "$baseUrl/api/v1/games/$gameId/draw" -Headers $headers -ContentType "application/json"
    Write-Host "  ✗ Deveria ter falhado!" -ForegroundColor Red
}
catch {
    Write-Host "  ✓ Erro esperado: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Green
}
Write-Host ""

# Tentar criar jogo com 0 jogadores
Write-Host "► Tentando criar jogo sem jogadores (deve falhar)" -ForegroundColor Yellow
$badGameBody = @{ emails = @() } | ConvertTo-Json
try {
    $null = Invoke-RestMethod -Method POST -Uri "$baseUrl/api/v1/games" -Body $badGameBody -ContentType "application/json"
    Write-Host "  ✗ Deveria ter falhado!" -ForegroundColor Red
}
catch {
    Write-Host "  ✓ Erro esperado: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Green
}
Write-Host ""

# Tentar login com jogador inexistente
Write-Host "► Tentando login com jogador inexistente (deve falhar)" -ForegroundColor Yellow
$badLoginBody = @{
    email = "hacker@example.com"
    game_id = $gameId
} | ConvertTo-Json
try {
    $null = Invoke-RestMethod -Method POST -Uri "$baseUrl/api/v1/auth/login" -Body $badLoginBody -ContentType "application/json"
    Write-Host "  ✗ Deveria ter falhado!" -ForegroundColor Red
}
catch {
    Write-Host "  ✓ Erro esperado: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Green
}
Write-Host ""

# Summary
Write-Host "======================================" -ForegroundColor Cyan
Write-Host "Testes Concluídos!" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Game ID usado: $gameId" -ForegroundColor Yellow
Write-Host ""
Write-Host "Para mais testes, use:" -ForegroundColor Gray
Write-Host "  - Postman: Importe Blackjack_API.postman_collection.json" -ForegroundColor Gray
Write-Host "  - VS Code REST Client: Use api_tests.http" -ForegroundColor Gray
Write-Host "  - Este script: .\test_api.ps1" -ForegroundColor Gray
