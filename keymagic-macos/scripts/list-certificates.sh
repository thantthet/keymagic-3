#!/bin/bash
# List Apple Developer certificates on macOS

echo "=== Apple Developer Certificates ==="
echo

# List all codesigning identities
echo "Code Signing Identities:"
echo "------------------------"
security find-identity -v -p codesigning | grep -E "Developer ID|Apple Development|Mac Developer|3rd Party Mac Developer"

echo
echo "=== Detailed Certificate Information ==="
echo

# List Developer ID Application certificates
echo "Developer ID Application certificates:"
echo "-------------------------------------"
security find-certificate -c "Developer ID Application" -p -Z | grep -E "SHA-1|Subject:|Issuer:|Not Valid"

echo
echo "Developer ID Installer certificates:"
echo "-----------------------------------"
security find-certificate -c "Developer ID Installer" -p -Z | grep -E "SHA-1|Subject:|Issuer:|Not Valid"

echo
echo "=== Quick Reference ==="
echo
echo "To use for signing, copy the full string in quotes, for example:"
echo '  "Developer ID Application: Your Name (TEAMID)"'
echo
echo "To check a specific certificate:"
echo '  security find-certificate -c "Developer ID Application: Your Name"'
echo
echo "To export a certificate:"
echo '  security export -t certs -f pemseq -k ~/Library/Keychains/login.keychain-db -o cert.pem'