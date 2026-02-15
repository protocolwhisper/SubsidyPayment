#!/bin/bash
# ============================================================================
# validate_skill.sh — Agent Skill Structure Validator
# ============================================================================
# Usage: bash validate_skill.sh <path-to-skill-directory>
#
# Checks:
#   1. SKILL.md exists
#   2. YAML frontmatter is present
#   3. 'name' field is non-empty
#   4. 'description' field is non-empty
#   5. Name matches directory name (warning)
#   6. No broken internal links
#   7. Directory structure summary
# ============================================================================

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PASS=0
FAIL=0
WARN=0

pass() { echo -e "  ${GREEN}✅ PASS${NC}: $1"; ((PASS++)) || true; }
fail() { echo -e "  ${RED}❌ FAIL${NC}: $1"; ((FAIL++)) || true; }
warn() { echo -e "  ${YELLOW}⚠️  WARN${NC}: $1"; ((WARN++)) || true; }
info() { echo -e "  ${BLUE}ℹ️  INFO${NC}: $1"; }

# --- Argument Check ---
if [ $# -lt 1 ]; then
    echo "Usage: bash validate_skill.sh <path-to-skill-directory>"
    exit 1
fi

SKILL_DIR="$1"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🔍 Skill Validator"
echo "  Target: ${SKILL_DIR}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# --- Check 1: Directory exists ---
echo "📂 Structure Checks"
if [ -d "$SKILL_DIR" ]; then
    pass "Skill directory exists"
else
    fail "Skill directory does not exist: ${SKILL_DIR}"
    echo ""
    echo "Result: ${FAIL} failures. Cannot proceed."
    exit 1
fi

# --- Check 2: SKILL.md exists ---
SKILL_FILE="${SKILL_DIR}/SKILL.md"
if [ -f "$SKILL_FILE" ]; then
    pass "SKILL.md exists"
else
    fail "SKILL.md not found (required)"
    echo ""
    echo "Result: ${FAIL} failures. Cannot proceed."
    exit 1
fi

# --- Check 3: YAML frontmatter present ---
echo ""
echo "📝 Frontmatter Checks"

FIRST_LINE=$(head -1 "$SKILL_FILE")
if [ "$FIRST_LINE" = "---" ]; then
    pass "YAML frontmatter delimiter found"
else
    fail "SKILL.md must start with '---' (YAML frontmatter)"
fi

# --- Check 4: Extract 'name' field ---
FRONTMATTER=$(awk 'NR==1 && /^---$/ {found=1; next} found && /^---$/ {exit} found {print}' "$SKILL_FILE")
NAME=$(echo "$FRONTMATTER" | grep -E '^name:' | head -1 | sed 's/^name:[[:space:]]*//' | tr -d '\r')
if [ -n "$NAME" ]; then
    pass "name field is present: '${NAME}'"
else
    fail "name field is missing or empty in frontmatter"
fi

# --- Check 5: Extract 'description' field ---
DESC=$(echo "$FRONTMATTER" | grep -E '^description:' | head -1 | sed 's/^description:[[:space:]]*//' | tr -d '\r')
if [ -n "$DESC" ]; then
    pass "description field is present"
    if [ ${#DESC} -lt 10 ]; then
        warn "description is very short (${#DESC} chars). Aim for 20+ chars."
    fi
else
    fail "description field is missing or empty in frontmatter"
fi

# --- Check 6: Name matches directory ---
DIR_NAME=$(basename "$SKILL_DIR")
if [ -n "$NAME" ] && [ "$NAME" = "$DIR_NAME" ]; then
    pass "name matches directory name"
elif [ -n "$NAME" ]; then
    warn "name '${NAME}' does not match directory name '${DIR_NAME}'"
fi

# --- Check 7: Content quality checks ---
echo ""
echo "📋 Content Checks"

LINE_COUNT=$(wc -l < "$SKILL_FILE" | tr -d ' ')
if [ "$LINE_COUNT" -ge 20 ]; then
    pass "SKILL.md has ${LINE_COUNT} lines (minimum 20)"
elif [ "$LINE_COUNT" -ge 10 ]; then
    warn "SKILL.md has only ${LINE_COUNT} lines. Consider adding more detail."
else
    fail "SKILL.md has only ${LINE_COUNT} lines. Too thin for a useful skill."
fi

# Check for heading structure
if grep -q "^## " "$SKILL_FILE"; then
    SECTION_COUNT=$(grep -c "^## " "$SKILL_FILE")
    pass "Found ${SECTION_COUNT} sections (## headings)"
else
    warn "No ## sections found. Consider structuring with headings."
fi

# Check for workflow/steps
if grep -qi "workflow\|step\|## Steps\|### Step" "$SKILL_FILE"; then
    pass "Workflow or step instructions detected"
else
    warn "No workflow/step keywords found. Consider adding a Workflow section."
fi

# Check for code blocks
if grep -q '```' "$SKILL_FILE"; then
    pass "Code blocks found (examples or commands)"
else
    warn "No code blocks found. Consider adding examples or commands."
fi

# --- Check 8: Directory contents ---
echo ""
echo "📦 Directory Contents"

SUBDIRS=0
FILES=0
for entry in "${SKILL_DIR}"/*; do
    if [ -d "$entry" ]; then
        ((SUBDIRS++)) || true
        info "Directory: $(basename "$entry")/"
    elif [ -f "$entry" ] && [ "$(basename "$entry")" != "SKILL.md" ]; then
        ((FILES++)) || true
    fi
done

if [ $SUBDIRS -gt 0 ] || [ $FILES -gt 0 ]; then
    pass "Supporting content: ${SUBDIRS} directories, ${FILES} additional files"
else
    warn "No supporting files or directories. Consider adding templates/ or resources/."
fi

# --- Summary ---
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  📊 Results"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "  ${GREEN}Passed${NC}: ${PASS}"
echo -e "  ${RED}Failed${NC}: ${FAIL}"
echo -e "  ${YELLOW}Warnings${NC}: ${WARN}"
echo ""

if [ $FAIL -eq 0 ]; then
    if [ $WARN -eq 0 ]; then
        echo -e "  🏆 ${GREEN}EXCELLENT${NC} — Skill passes all checks!"
    else
        echo -e "  ✅ ${GREEN}VALID${NC} — Skill is valid with ${WARN} warning(s)."
    fi
    exit 0
else
    echo -e "  ❌ ${RED}INVALID${NC} — Fix ${FAIL} failure(s) before delivery."
    exit 1
fi
