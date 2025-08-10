#!/bin/bash

# Test script for password reset functionality
# Make sure the server is running: cargo run

BASE_URL="http://localhost:3001"

echo "🧪 Testing Password Reset Functionality"
echo "======================================"

# Test 1: Register a test user
echo "1️⃣  Registering test user..."
REGISTER_RESPONSE=$(curl -s -X POST $BASE_URL/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test User",
    "email": "test@example.com",
    "phone": "+1234567890",
    "password": "TestPassword123!",
    "password_confirmation": "TestPassword123!"
  }')

echo "Register Response: $REGISTER_RESPONSE"
echo ""

# Test 2: Request password reset
echo "2️⃣  Requesting password reset..."
FORGOT_RESPONSE=$(curl -s -X POST $BASE_URL/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com"
  }')

echo "Forgot Password Response: $FORGOT_RESPONSE"
echo ""

# Test 3: Try with invalid email
echo "3️⃣  Testing with invalid email..."
INVALID_RESPONSE=$(curl -s -X POST $BASE_URL/auth/forgot-password \
  -H "Content-Type: application/json" \
  -d '{
    "email": "nonexistent@example.com"
  }')

echo "Invalid Email Response: $INVALID_RESPONSE"
echo ""

# Test 4: Test reset password with invalid OTP
echo "4️⃣  Testing password reset with invalid OTP..."
RESET_RESPONSE=$(curl -s -X POST $BASE_URL/auth/reset-password \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "otp": "000000",
    "new_password": "NewPassword123!",
    "confirm_password": "NewPassword123!"
  }')

echo "Reset with Invalid OTP Response: $RESET_RESPONSE"
echo ""

echo "✅ Password reset API tests completed!"
echo ""
echo "📧 Note: To fully test the functionality:"
echo "   1. Configure real SMTP settings in .env"
echo "   2. Use a real email address"
echo "   3. Check your email for the OTP"
echo "   4. Use the received OTP in the reset-password endpoint"
