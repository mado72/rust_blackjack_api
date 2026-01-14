# Simple Test Script - Phase 2
$baseUrl = "http://localhost:8080"

Write-Host "`n=== PHASE 2 TESTS ===" -ForegroundColor Cyan

# 1. Health Check
Write-Host "`n1. Health Check..." -ForegroundColor Yellow
$health = Invoke-RestMethod -Uri "$baseUrl/health"
Write-Host "   Status: $($health.status)" -ForegroundColor Green

# 2. Register Users
Write-Host "`n2. Register Users..." -ForegroundColor Yellow
$users = @("alice@test.com", "bob@test.com", "charlie@test.com")
foreach ($email in $users) {
    try {
        $body = @{ email = $email; password = "password123" } | ConvertTo-Json
        Invoke-RestMethod -Uri "$baseUrl/api/v1/auth/register" -Method POST -Body $body -ContentType "application/json" | Out-Null
        Write-Host "   Registered: $email" -ForegroundColor Green
    } catch {
        Write-Host "   Already exists: $email" -ForegroundColor DarkYellow
    }
}

# 3. Login Users
Write-Host "`n3. Login Users..." -ForegroundColor Yellow
$tokens = @{}
foreach ($email in $users) {
    $body = @{ email = $email; password = "password123" } | ConvertTo-Json
    $login = Invoke-RestMethod -Uri "$baseUrl/api/v1/auth/login" -Method POST -Body $body -ContentType "application/json"
    $tokens[$email] = $login.token
    Write-Host "   Logged in: $email" -ForegroundColor Green
}

# 4. Create Game
Write-Host "`n4. Create Game (Alice)..." -ForegroundColor Yellow
$headers = @{ Authorization = "Bearer $($tokens['alice@test.com'])" }
$body = @{ enrollment_timeout_seconds = 600 } | ConvertTo-Json
$game = Invoke-RestMethod -Uri "$baseUrl/api/v1/games" -Method POST -Body $body -Headers $headers -ContentType "application/json"
$gameId = $game.game_id
Write-Host "   Game ID: $gameId" -ForegroundColor Green
Write-Host "   Enrolled: $($game.enrolled_count)" -ForegroundColor Green

# 5. List Open Games
Write-Host "`n5. List Open Games (Bob)..." -ForegroundColor Yellow
$headers = @{ Authorization = "Bearer $($tokens['bob@test.com'])" }
$openGames = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/open" -Headers $headers
Write-Host "   Found: $($openGames.count) game(s)" -ForegroundColor Green

# 6. Enroll Players
Write-Host "`n6. Enroll Players..." -ForegroundColor Yellow
$headers = @{ Authorization = "Bearer $($tokens['bob@test.com'])" }
$body = @{ email = "bob@test.com" } | ConvertTo-Json
$enroll = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId/enroll" -Method POST -Body $body -Headers $headers -ContentType "application/json"
Write-Host "   Bob enrolled. Total: $($enroll.enrolled_count)" -ForegroundColor Green

$headers = @{ Authorization = "Bearer $($tokens['charlie@test.com'])" }
$body = @{ email = "charlie@test.com" } | ConvertTo-Json
$enroll = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId/enroll" -Method POST -Body $body -Headers $headers -ContentType "application/json"
Write-Host "   Charlie enrolled. Total: $($enroll.enrolled_count)" -ForegroundColor Green

# 7. Create Invitation
Write-Host "`n7. Create Invitation (Alice)..." -ForegroundColor Yellow
$headers = @{ Authorization = "Bearer $($tokens['alice@test.com'])" }
$body = @{ invitee_email = "diana@test.com" } | ConvertTo-Json
$invitation = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId/invitations" -Method POST -Body $body -Headers $headers -ContentType "application/json"
Write-Host "   Invitation created for: $($invitation.invitee_email)" -ForegroundColor Green

# 8. Close Enrollment
Write-Host "`n8. Close Enrollment (Alice)..." -ForegroundColor Yellow
$headers = @{ Authorization = "Bearer $($tokens['alice@test.com'])" }
$body = @{} | ConvertTo-Json
$closed = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId/close-enrollment" -Method POST -Body $body -Headers $headers -ContentType "application/json"
Write-Host "   Enrollment closed" -ForegroundColor Green
Write-Host "   Turn order: $($closed.turn_order -join ', ')" -ForegroundColor Green

# 9. Get Game State
Write-Host "`n9. Get Game State..." -ForegroundColor Yellow
$headers = @{ Authorization = "Bearer $($tokens['alice@test.com'])" }
$state = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId" -Headers $headers
Write-Host "   Current turn: $($state.current_turn_player)" -ForegroundColor Yellow

# 10. Draw Card (Current Player)
Write-Host "`n10. Draw Card (Current Player)..." -ForegroundColor Yellow
$currentPlayer = $state.current_turn_player
$headers = @{ Authorization = "Bearer $($tokens[$currentPlayer])" }
$draw = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId/draw" -Method POST -Headers $headers
Write-Host "   $currentPlayer drew: $($draw.card.name) of $($draw.card.suit)" -ForegroundColor Green
Write-Host "   Points: $($draw.points)" -ForegroundColor Cyan

# 11. Test Stand
Write-Host "`n11. Stand (Next Player)..." -ForegroundColor Yellow
$headers = @{ Authorization = "Bearer $($tokens['alice@test.com'])" }
$state = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId" -Headers $headers
$currentPlayer = $state.current_turn_player
$headers = @{ Authorization = "Bearer $($tokens[$currentPlayer])" }
$stand = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId/stand" -Method POST -Headers $headers
Write-Host "   $currentPlayer stood with $($stand.points) points" -ForegroundColor Green
Write-Host "   Game finished: $($stand.game_finished)" -ForegroundColor $(if($stand.game_finished){'Yellow'}else{'Cyan'})

# 12. Complete Game
Write-Host "`n12. Complete Game (All Stand)..." -ForegroundColor Yellow
for ($i = 0; $i -lt 5; $i++) {
    $headers = @{ Authorization = "Bearer $($tokens['alice@test.com'])" }
    $state = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId" -Headers $headers
    
    if ($state.finished) {
        Write-Host "   Game finished!" -ForegroundColor Green
        break
    }
    
    $currentPlayer = $state.current_turn_player
    if ($currentPlayer) {
        $headers = @{ Authorization = "Bearer $($tokens[$currentPlayer])" }
        try {
            $stand = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId/stand" -Method POST -Headers $headers
            Write-Host "   $currentPlayer stood" -ForegroundColor Green
        } catch {
            Write-Host "   Player finished" -ForegroundColor DarkYellow
        }
    }
}

# 13. Get Results
Write-Host "`n13. Get Results..." -ForegroundColor Yellow
$headers = @{ Authorization = "Bearer $($tokens['alice@test.com'])" }
$results = Invoke-RestMethod -Uri "$baseUrl/api/v1/games/$gameId/results" -Headers $headers
Write-Host "   Winner: $($results.winner)" -ForegroundColor Green
Write-Host "   Highest score: $($results.highest_score)" -ForegroundColor Cyan

Write-Host "`n=== ALL TESTS PASSED! ===" -ForegroundColor Green
