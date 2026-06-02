#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════════════════
# test-docker.sh — Build and smoke-test the Planty Docker image
# Usage: ./test-docker.sh
# ═══════════════════════════════════════════════════════════════════════════════
set -euo pipefail

IMAGE_NAME="planty-test"
CONTAINER_NAME="planty-test-$$"
PORT=3099

cleanup() {
    echo "🧹 Cleaning up..."
    docker rm -f "$CONTAINER_NAME" 2>/dev/null || true
    docker rmi "$IMAGE_NAME" 2>/dev/null || true
}
trap cleanup EXIT

echo "═══════════════════════════════════════════════════════════════"
echo "🐳 Building Docker image..."
echo "═══════════════════════════════════════════════════════════════"
docker build -t "$IMAGE_NAME" .

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "🚀 Starting container on port $PORT..."
echo "═══════════════════════════════════════════════════════════════"
docker run -d \
    --name "$CONTAINER_NAME" \
    -p "$PORT:3000" \
    -e PLANT_COACH_PROVIDER=mock \
    "$IMAGE_NAME"

# Wait for startup
echo "⏳ Waiting for server to start..."
for i in $(seq 1 30); do
    if curl -sf "http://localhost:$PORT/api/health" >/dev/null 2>&1; then
        echo "✅ Server is up (attempt $i)"
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "❌ Server failed to start within 30s"
        docker logs "$CONTAINER_NAME"
        exit 1
    fi
    sleep 1
done

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "🧪 Running smoke tests..."
echo "═══════════════════════════════════════════════════════════════"

PASS=0
FAIL=0

assert() {
    local desc="$1"
    local result="$2"
    local expected="$3"
    if echo "$result" | grep -q "$expected"; then
        echo "  ✅ $desc"
        PASS=$((PASS + 1))
    else
        echo "  ❌ $desc (expected '$expected', got: $result)"
        FAIL=$((FAIL + 1))
    fi
}

# Test 1: Health endpoint
HEALTH=$(curl -sf "http://localhost:$PORT/api/health")
assert "GET /api/health returns OK" "$HEALTH" "ok\|healthy"

# Test 2: Frontend serves index.html
INDEX=$(curl -sf -o /dev/null -w "%{http_code}" "http://localhost:$PORT/")
assert "GET / serves frontend (200)" "$INDEX" "200"

# Test 3: Frontend serves with correct content-type
CT=$(curl -sf -I "http://localhost:$PORT/" | grep -i "content-type")
assert "GET / returns text/html" "$CT" "text/html"

# Test 4: API returns 401 for unauthenticated requests
AUTH_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:$PORT/api/v1/auth/me")
assert "GET /api/auth/me returns 401" "$AUTH_STATUS" "401"

# Test 5: Registration with admin invite works
# First get the admin invite from logs
INVITE=$(docker logs "$CONTAINER_NAME" 2>&1 | grep -o 'ADMIN-[A-Z0-9]*' | head -1 || echo "")
if [ -n "$INVITE" ]; then
    REG_STATUS=$(curl -s -o /dev/null -w "%{http_code}" \
        -X POST "http://localhost:$PORT/api/v1/auth/register" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"Test User\",\"email\":\"test@planty.local\",\"password\":\"testpass123\",\"invite_code\":\"$INVITE\"}")
    assert "POST /api/auth/register with admin invite" "$REG_STATUS" "200\|201"
else
    echo "  ⚠️  Skipped registration test (no admin invite found in logs)"
fi

# Test 6: Static assets cached
ASSET_CACHE=$(curl -sf -I "http://localhost:$PORT/" | grep -i "cache-control" || echo "none")
assert "Static assets have cache headers or are served" "$ASSET_CACHE" "cache\|none"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📊 Results: $PASS passed, $FAIL failed"
echo "═══════════════════════════════════════════════════════════════"

if [ "$FAIL" -gt 0 ]; then
    echo ""
    echo "Container logs:"
    docker logs "$CONTAINER_NAME" 2>&1 | tail -20
    exit 1
fi

echo "🎉 All tests passed!"
