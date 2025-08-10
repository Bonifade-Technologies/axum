#!/bin/bash

# Performance Test: Password Reset with Apalis Job Queue
# This script measures the response time for password reset operations

BASE_URL="http://localhost:3001"
EMAIL="perf.test.$(date +%s)@example.com"
PASSWORD="TestPassword123!"
NEW_PASSWORD="NewTestPassword123!"

echo "=== Password Reset Performance Test ==="
echo "Email: $EMAIL"
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}→ $1${NC}"
}

# Function to measure execution time
measure_time() {
    local start_time=$(date +%s%N)
    "$@"
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
    echo "$duration"
}

# Test 1: Register user
print_step "Step 1: User Registration"
REGISTER_TIME=$(measure_time curl -s -o /dev/null -w "%{time_total}" -X POST "$BASE_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Performance Test User\",
    \"email\": \"$EMAIL\",
    \"phone\": \"+1234567890\",
    \"password\": \"$PASSWORD\",
    \"confirm_password\": \"$PASSWORD\"
  }")

REGISTER_MS=$(echo "$REGISTER_TIME * 1000" | bc -l | cut -d. -f1)
print_info "Registration took: ${REGISTER_MS}ms"
echo

# Test 2: Request password reset (measure time)
print_step "Step 2: Forgot Password Request"
FORGOT_TIME=$(measure_time curl -s -o /dev/null -w "%{time_total}" -X POST "$BASE_URL/auth/forgot-password" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$EMAIL\"
  }")

FORGOT_MS=$(echo "$FORGOT_TIME * 1000" | bc -l | cut -d. -f1)
print_info "Forgot password request took: ${FORGOT_MS}ms"
echo

# Test 3: Reset password with dummy OTP (measure time)
print_step "Step 3: Password Reset Performance Test"
print_info "Using test OTP: 123456"

# Measure the password reset operation
start_time=$(date +%s%N)
RESET_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}\nTIME_TOTAL:%{time_total}" -X POST "$BASE_URL/auth/reset-password" \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$EMAIL\",
    \"otp\": \"123456\",
    \"new_password\": \"$NEW_PASSWORD\",
    \"confirm_password\": \"$NEW_PASSWORD\"
  }")
end_time=$(date +%s%N)

# Extract timing information
HTTP_STATUS=$(echo "$RESET_RESPONSE" | grep "HTTP_STATUS" | cut -d':' -f2)
TIME_TOTAL=$(echo "$RESET_RESPONSE" | grep "TIME_TOTAL" | cut -d':' -f2)
RESPONSE_BODY=$(echo "$RESET_RESPONSE" | sed '/HTTP_STATUS/d' | sed '/TIME_TOTAL/d')

RESET_MS=$(echo "$TIME_TOTAL * 1000" | bc -l | cut -d. -f1)
MANUAL_MS=$(( (end_time - start_time) / 1000000 ))

echo "HTTP Status: $HTTP_STATUS"
echo "Response: $RESPONSE_BODY"
echo
print_info "Password reset HTTP response time: ${RESET_MS}ms"
print_info "Manual measurement: ${MANUAL_MS}ms"

# Performance analysis
echo
print_step "Performance Analysis"

if [ "$RESET_MS" -lt 200 ]; then
    print_success "Excellent performance: ${RESET_MS}ms (< 200ms)"
    print_info "✅ Job queue is working properly - response returned before email processing"
elif [ "$RESET_MS" -lt 500 ]; then
    print_success "Good performance: ${RESET_MS}ms (< 500ms)"
    print_info "✅ Acceptable response time with background processing"
elif [ "$RESET_MS" -lt 1000 ]; then
    print_info "Fair performance: ${RESET_MS}ms (< 1s)"
    print_info "⚠️  Response time could be improved"
else
    print_error "Poor performance: ${RESET_MS}ms (> 1s)"
    print_info "❌ Job might not be running in background properly"
fi

echo
print_step "Background Job Status"
print_info "Check server logs for Apalis job processing messages:"
print_info "  📤 [Apalis] Queuing password reset success email"
print_info "  🔄 [Apalis Worker] Processing password reset success email"
print_info "  ✅ [Apalis Worker] Password reset success email sent"

echo
print_step "Expected Behavior"
echo "1. HTTP response should return in <200ms"
echo "2. Email job should be queued in Redis"
echo "3. Apalis worker should process email in background"
echo "4. User receives email confirmation separately"

echo
print_success "Performance test completed!"
