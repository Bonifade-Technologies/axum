#!/bin/bash

# Test script for rate limiting functionality
# Make sure the server is running: cargo run

BASE_URL="http://localhost:3001"
TEST_EMAIL="bowofadeoyerinde@gmail.com"

echo "🧪 Testing Rate Limiting for Forgot Password"
echo "==========================================="

# Test 1: First request (should succeed)
echo "1️⃣  First forgot password request (should succeed)..."
RESPONSE1=$(curl -s -X POST $BASE_URL/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$TEST_EMAIL\"}")

echo "Response 1: $RESPONSE1"
echo ""

# Test 2: Immediate second request (should be rate limited)
echo "2️⃣  Immediate second request (should be rate limited)..."
RESPONSE2=$(curl -s -X POST $BASE_URL/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$TEST_EMAIL\"}")

echo "Response 2: $RESPONSE2"
echo ""

# Test 3: Third request (should still be rate limited)
echo "3️⃣  Third request (should still be rate limited)..."
RESPONSE3=$(curl -s -X POST $BASE_URL/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$TEST_EMAIL\"}")

echo "Response 3: $RESPONSE3"
echo ""

# Test 4: Different email (should succeed)
echo "4️⃣  Different email test (should succeed)..."
RESPONSE4=$(curl -s -X POST $BASE_URL/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{"email": "different@example.com"}')

echo "Response 4: $RESPONSE4"
echo ""

echo "✅ Rate limiting test completed!"
echo ""
echo "📊 Expected Results:"
echo "   ✅ Response 1: Success with OTP sent"
echo "   🚫 Response 2: Rate limited (429 status)"
echo "   🚫 Response 3: Rate limited (429 status)"
echo "   ❓ Response 4: Should fail (email not found) but not rate limited"
echo ""
echo "⏰ Rate limit: 5 minutes per email address"
echo "🔄 To test successful retry, wait 5 minutes and run again"
