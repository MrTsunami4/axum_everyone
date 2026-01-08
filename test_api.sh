#!/bin/bash

# Bash script to test the Axum API

BASE_URL="http://localhost:3000"

# Color codes
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function print_header() {
    echo -e "\n${GREEN}========================================${NC}"
    echo -e "${GREEN}$1${NC}"
    echo -e "${GREEN}========================================${NC}\n"
}

function test_root() {
    print_header "Test 1: GET /"
    response=$(curl -s -w "\n%{http_code}" "$BASE_URL/")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    echo -e "${CYAN}Status: $http_code${NC}"
    echo -e "${NC}Response: $body${NC}"
}

function test_get_all_jokes() {
    print_header "Test 2: GET /jokes (Get all jokes)"
    response=$(curl -s -w "\n%{http_code}" "$BASE_URL/jokes")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    echo -e "${CYAN}Status: $http_code${NC}"
    echo -e "${NC}Response: $body${NC}"
}

function test_create_joke() {
    local content="$1"
    print_header "Test 3: POST /jokes (Create a joke)"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/jokes" \
        -H "Content-Type: application/json" \
        -d "{\"content\":\"$content\"}")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    echo -e "${CYAN}Status: $http_code${NC}"
    echo -e "${NC}Response: $body${NC}"
    # Extract ID from response
    id=$(echo "$body" | grep -o '"id":[0-9]*' | head -1 | grep -o '[0-9]*')
    echo "$id"
}

function test_get_joke() {
    local id="$1"
    print_header "Test 4: GET /joke/{id} (Get specific joke)"
    response=$(curl -s -w "\n%{http_code}" "$BASE_URL/joke/$id")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    echo -e "${CYAN}Status: $http_code${NC}"
    echo -e "${NC}Response: $body${NC}"
}

function test_delete_joke() {
    local id="$1"
    print_header "Test 5: DELETE /joke/{id} (Delete specific joke)"
    response=$(curl -s -w "\n%{http_code}" -X DELETE "$BASE_URL/joke/$id")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    echo -e "${CYAN}Status: $http_code${NC}"
    echo -e "${NC}Response: $body${NC}"
}

function test_delete_all_jokes() {
    print_header "Test 6: DELETE /jokes (Delete all jokes)"
    response=$(curl -s -w "\n%{http_code}" -X DELETE "$BASE_URL/jokes")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    echo -e "${CYAN}Status: $http_code${NC}"
    echo -e "${NC}Response: $body${NC}"
}

# Main execution
echo -e "${YELLOW}Starting API Tests...${NC}"
echo -e "${YELLOW}Base URL: $BASE_URL${NC}"

test_root
test_get_all_jokes

joke_id1=$(test_create_joke "Why did the programmer quit his job? Because he didn't get arrays." | tail -1)
if [ -n "$joke_id1" ]; then
    test_get_joke "$joke_id1"
fi

joke_id2=$(test_create_joke "How many programmers does it take to change a light bulb? None, that's a hardware problem." | tail -1)
if [ -n "$joke_id2" ]; then
    test_get_joke "$joke_id2"
fi

test_get_all_jokes

if [ -n "$joke_id1" ]; then
    test_delete_joke "$joke_id1"
fi

test_get_all_jokes
test_delete_all_jokes
test_get_all_jokes

echo -e "\n${YELLOW}API Tests Complete!${NC}\n"
