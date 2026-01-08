# PowerShell script to test the Axum API

$BASE_URL = "http://localhost:3000"

function Write-Header {
    param([string]$Text)
    Write-Host "`n========================================" -ForegroundColor Green
    Write-Host $Text -ForegroundColor Green
    Write-Host "========================================`n" -ForegroundColor Green
}

function Test-Root {
    Write-Header "Test 1: GET /"
    try {
        $response = Invoke-WebRequest -Uri "$BASE_URL/" -Method Get
        Write-Host "Status: $($response.StatusCode)" -ForegroundColor Cyan
        Write-Host "Response: $($response.Content)" -ForegroundColor White
    }
    catch {
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    }
}

function Test-GetAllJokes {
    Write-Header "Test 2: GET /jokes (Get all jokes)"
    try {
        $response = Invoke-WebRequest -Uri "$BASE_URL/jokes" -Method Get
        Write-Host "Status: $($response.StatusCode)" -ForegroundColor Cyan
        $content = $response.Content | ConvertFrom-Json
        Write-Host "Response: $($content | ConvertTo-Json -Depth 10)" -ForegroundColor White
    }
    catch {
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    }
}

function Test-CreateJoke {
    param([string]$Content)
    Write-Header "Test 3: POST /jokes (Create a joke)"
    try {
        $body = @{ content = $Content } | ConvertTo-Json
        $response = Invoke-WebRequest -Uri "$BASE_URL/jokes" `
            -Method Post `
            -ContentType "application/json" `
            -Body $body
        Write-Host "Status: $($response.StatusCode)" -ForegroundColor Cyan
        $jsonResponse = $response.Content | ConvertFrom-Json
        Write-Host "Response: $($response.Content)" -ForegroundColor White
        return $jsonResponse.id
    }
    catch {
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

function Test-GetJoke {
    param([int64]$Id)
    Write-Header "Test 4: GET /joke/{id} (Get specific joke)"
    try {
        $response = Invoke-WebRequest -Uri "$BASE_URL/joke/$Id" -Method Get
        Write-Host "Status: $($response.StatusCode)" -ForegroundColor Cyan
        Write-Host "Response: $($response.Content)" -ForegroundColor White
    }
    catch {
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    }
}

function Test-DeleteJoke {
    param([int64]$Id)
    Write-Header "Test 5: DELETE /joke/{id} (Delete specific joke)"
    try {
        $response = Invoke-WebRequest -Uri "$BASE_URL/joke/$Id" -Method Delete
        Write-Host "Status: $($response.StatusCode)" -ForegroundColor Cyan
        Write-Host "Response: $($response.Content)" -ForegroundColor White
    }
    catch {
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    }
}

function Test-DeleteAllJokes {
    Write-Header "Test 6: DELETE /jokes (Delete all jokes)"
    try {
        $response = Invoke-WebRequest -Uri "$BASE_URL/jokes" -Method Delete
        Write-Host "Status: $($response.StatusCode)" -ForegroundColor Cyan
        Write-Host "Response: $($response.Content)" -ForegroundColor White
    }
    catch {
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Main execution
Write-Host "Starting API Tests..." -ForegroundColor Yellow
Write-Host "Base URL: $BASE_URL" -ForegroundColor Yellow

Test-Root
Test-GetAllJokes

$jokeId1 = Test-CreateJoke "Why did the programmer quit his job? Because he didn't get arrays."
if ($jokeId1) {
    Test-GetJoke $jokeId1
}

$jokeId2 = Test-CreateJoke "How many programmers does it take to change a light bulb? None, that's a hardware problem."
if ($jokeId2) {
    Test-GetJoke $jokeId2
}

Test-GetAllJokes

if ($jokeId1) {
    Test-DeleteJoke $jokeId1
}

Test-GetAllJokes
Test-DeleteAllJokes
Test-GetAllJokes

Write-Host "`nAPI Tests Complete!`n" -ForegroundColor Yellow
