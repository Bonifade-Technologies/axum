#!/bin/bash

# SMTP Connection Test Script
# Tests SMTP server connectivity before running the full application

echo "=== SMTP Connection Test ==="
echo

# Read SMTP settings from .env file
if [ -f ".env" ]; then
    source .env
    echo "📧 SMTP Configuration from .env:"
    echo "   Host: $SMTP_HOST"
    echo "   Port: $SMTP_PORT"
    echo "   Username: $SMTP_USERNAME"
    echo "   From Email: $FROM_EMAIL"
    echo
else
    echo "❌ .env file not found!"
    exit 1
fi

# Test 1: Basic connectivity
echo "🔍 Test 1: Basic connectivity to SMTP server"
if command -v telnet >/dev/null 2>&1; then
    echo "Testing connection to $SMTP_HOST:$SMTP_PORT..."
    timeout 5 telnet $SMTP_HOST $SMTP_PORT 2>/dev/null
    if [ $? -eq 0 ]; then
        echo "✅ Basic connection successful"
    else
        echo "❌ Connection failed - server might be down or port blocked"
    fi
else
    echo "⚠️  telnet not available, skipping basic connectivity test"
fi
echo

# Test 2: DNS resolution
echo "🔍 Test 2: DNS resolution"
if command -v nslookup >/dev/null 2>&1; then
    echo "Resolving $SMTP_HOST..."
    nslookup $SMTP_HOST
    if [ $? -eq 0 ]; then
        echo "✅ DNS resolution successful"
    else
        echo "❌ DNS resolution failed"
    fi
else
    echo "⚠️  nslookup not available, skipping DNS test"
fi
echo

# Test 3: Port check with nc (netcat)
echo "🔍 Test 3: Port connectivity check"
if command -v nc >/dev/null 2>&1; then
    echo "Testing port $SMTP_PORT on $SMTP_HOST..."
    timeout 5 nc -z $SMTP_HOST $SMTP_PORT
    if [ $? -eq 0 ]; then
        echo "✅ Port $SMTP_PORT is open and reachable"
    else
        echo "❌ Port $SMTP_PORT is not reachable"
        echo
        echo "🔧 Troubleshooting suggestions:"
        echo "   1. Check if the SMTP server is running"
        echo "   2. Verify the port number (common ports: 25, 465, 587, 2525)"
        echo "   3. Check firewall settings"
        echo "   4. Try alternative ports:"
        echo "      - Port 25 (standard SMTP)"
        echo "      - Port 465 (SMTP over SSL)"
        echo "      - Port 587 (SMTP with STARTTLS)"
        echo "      - Port 2525 (alternative SMTP)"
    fi
else
    echo "⚠️  netcat (nc) not available, skipping port test"
fi
echo

# Test 4: Alternative ports
echo "🔍 Test 4: Testing common SMTP ports"
COMMON_PORTS=(25 465 587 2525)

for port in "${COMMON_PORTS[@]}"; do
    if command -v nc >/dev/null 2>&1; then
        echo -n "  Port $port: "
        timeout 2 nc -z $SMTP_HOST $port 2>/dev/null
        if [ $? -eq 0 ]; then
            echo "✅ Open"
            if [ "$port" != "$SMTP_PORT" ]; then
                echo "    💡 Consider using port $port instead of $SMTP_PORT"
            fi
        else
            echo "❌ Closed"
        fi
    fi
done
echo

# Test 5: SMTP server banner (if port 25 is open)
echo "🔍 Test 5: SMTP server banner"
if command -v telnet >/dev/null 2>&1; then
    echo "Attempting to get SMTP banner from $SMTP_HOST:$SMTP_PORT..."
    (echo "QUIT"; sleep 1) | timeout 5 telnet $SMTP_HOST $SMTP_PORT 2>/dev/null | head -3
else
    echo "⚠️  telnet not available for banner test"
fi
echo

echo "=== Recommendations ==="
echo
echo "Based on your configuration:"
echo "🔧 SMTP Host: $SMTP_HOST"
echo "🔧 Current Port: $SMTP_PORT"
echo
echo "If connection fails, try:"
echo "1. Contact your hosting provider to confirm SMTP settings"
echo "2. Check if authentication is required"
echo "3. Verify SSL/TLS requirements"
echo "4. Test with different ports (25, 465, 587, 2525)"
echo "5. Check if your IP is whitelisted"
echo
echo "Common SMTP configurations:"
echo "• Port 587 with STARTTLS (most common)"
echo "• Port 465 with SSL/TLS"
echo "• Port 25 (often blocked by ISPs)"
echo "• Port 2525 (alternative for blocked port 25)"
