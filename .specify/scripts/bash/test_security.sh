#!/usr/bin/env bash
# Security test for get_feature_paths refactoring
# This test verifies that malicious branch names cannot execute arbitrary commands

set -e

SCRIPT_DIR="$(CDPATH="" cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

test_pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((TESTS_PASSED++))
}

test_fail() {
    echo -e "${RED}✗${NC} $1"
    ((TESTS_FAILED++))
}

run_test() {
    local test_name="$1"
    ((TESTS_RUN++))
    echo "Running: $test_name"
}

# Helper function to safely load variables from get_feature_paths
load_feature_paths() {
    while IFS= read -r line; do
        key=${line%%=*}
        value=${line#*=}
        printf -v "$key" '%s' "$value"
    done < <(get_feature_paths)
}

# Test 1: Malicious command substitution in branch name
run_test "Test command injection via \$() syntax"
PWNED_FILE="/tmp/pwned_$$"
rm -f "$PWNED_FILE"

export SPECIFY_FEATURE="001-feature'\$(touch $PWNED_FILE)'"

# Call get_feature_paths and load variables safely
# This should NOT create the pwned file
load_feature_paths

if [[ -f "$PWNED_FILE" ]]; then
    test_fail "Command injection vulnerability exists - file was created"
    rm -f "$PWNED_FILE"
else
    test_pass "No command injection via \$() syntax"
fi

# Test 2: Malicious backticks in branch name
run_test "Test backtick command injection"
rm -f "$PWNED_FILE"
export SPECIFY_FEATURE="001-feature\`touch $PWNED_FILE\`"

load_feature_paths

if [[ -f "$PWNED_FILE" ]]; then
    test_fail "Backtick command injection vulnerability exists"
    rm -f "$PWNED_FILE"
else
    test_pass "No backtick command injection"
fi

# Test 3: Single quote escape
run_test "Test single quote handling"
export SPECIFY_FEATURE="001-feature'test"
load_feature_paths

if [[ "$CURRENT_BRANCH" == "001-feature'test" ]]; then
    test_pass "Single quotes handled correctly"
else
    test_fail "Single quote handling broken: got '$CURRENT_BRANCH'"
fi

# Test 4: Double quote handling
run_test "Test double quote handling"
export SPECIFY_FEATURE='001-feature"test'
load_feature_paths

if [[ "$CURRENT_BRANCH" == '001-feature"test' ]]; then
    test_pass "Double quotes handled correctly"
else
    test_fail "Double quote handling broken: got '$CURRENT_BRANCH'"
fi

# Test 5: Semicolon handling
run_test "Test semicolon handling"
export SPECIFY_FEATURE="001-feature;test"
load_feature_paths

if [[ "$CURRENT_BRANCH" == "001-feature;test" ]]; then
    test_pass "Semicolons handled correctly"
else
    test_fail "Semicolon handling broken: got '$CURRENT_BRANCH'"
fi

# Test 6: Normal branch name still works
run_test "Test normal branch name"
export SPECIFY_FEATURE="001-normal-feature"
load_feature_paths

if [[ "$CURRENT_BRANCH" == "001-normal-feature" ]]; then
    test_pass "Normal branch names work correctly"
else
    test_fail "Normal branch name handling broken: got '$CURRENT_BRANCH'"
fi

# Test 7: Variables are properly set
run_test "Test all variables are set"
export SPECIFY_FEATURE="001-test-feature"
load_feature_paths

if [[ -n "$REPO_ROOT" ]] && [[ -n "$CURRENT_BRANCH" ]] && [[ -n "$FEATURE_DIR" ]] && \
   [[ -n "$FEATURE_SPEC" ]] && [[ -n "$IMPL_PLAN" ]]; then
    test_pass "All variables are set"
else
    test_fail "Some variables are not set"
fi

# Test 8: Variables contain expected paths
run_test "Test variable path structure"
export SPECIFY_FEATURE="001-path-test"
load_feature_paths

if [[ "$FEATURE_DIR" == "$REPO_ROOT/specs/001-path-test" ]] && \
   [[ "$FEATURE_SPEC" == "$FEATURE_DIR/spec.md" ]] && \
   [[ "$IMPL_PLAN" == "$FEATURE_DIR/plan.md" ]]; then
    test_pass "Variable paths are correctly structured"
else
    test_fail "Variable path structure is incorrect"
fi

# Cleanup
rm -f "$PWNED_FILE"

# Summary
echo
echo "================================"
echo "Test Summary"
echo "================================"
echo "Tests run: $TESTS_RUN"
echo -e "${GREEN}Tests passed: $TESTS_PASSED${NC}"
if [[ $TESTS_FAILED -gt 0 ]]; then
    echo -e "${RED}Tests failed: $TESTS_FAILED${NC}"
else
    echo "Tests failed: $TESTS_FAILED"
fi
echo "================================"

if [[ $TESTS_FAILED -gt 0 ]]; then
    exit 1
else
    exit 0
fi
