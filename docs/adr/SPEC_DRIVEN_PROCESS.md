# Spec-Driven ADR Process

This document defines how we develop new features using a spec-driven approach with Architecture Decision Records (ADRs).

## Overview

Spec-driven development combines planning and implementation through ADRs with explicit phase transitions:

```
Requirements → Design → Implementation Plan → Implementation → Accepted
```

Each phase requires explicit user approval before proceeding to the next.

## Process Rules v1.0

### Basic Principles
- **Interactive**: Wait for explicit user OK at each phase
- **User-Led**: User understanding and decision-making is essential
- **Incremental**: Progress one phase at a time only

### Phase Progression

#### 1. Requirements Phase
**Claude's Actions:**
1. Create ADR-XXX.md (Status: `Proposed: Requirements`)
2. Fill in Requirements section
3. Explicitly ask: "Please review the requirements. Are they OK? Any modifications needed?"
4. **WAIT** for user response

**User's Actions:**
- Read and understand requirements
- Respond with "OK" or request modifications

#### 2. Design Phase
**Claude's Actions:**
1. Update Status → `Proposed: Design`
2. Fill in Design section
3. Explicitly ask: "Please review the design. Is it OK? Any modifications needed?"
4. **WAIT** for user response

**User's Actions:**
- Read and understand design
- Respond with "OK" or request modifications

#### 3. Implementation Plan Phase
**Claude's Actions:**
1. Update Status → `Proposed: Plan`
2. Fill in Implementation Plan section
3. Explicitly ask: "Please review the implementation plan. Is it OK? Any modifications needed?"
4. **WAIT** for user response

**User's Actions:**
- Read and understand implementation plan
- Respond with "OK" or request modifications

#### 4. Implementation Phase
**Claude's Actions:**
1. **WAIT** for explicit user instruction to start implementation
2. Update Status → `In Progress`
3. Use TodoWrite tool to track implementation tasks
4. Follow Implementation Plan steps
5. Run quality gates (cargo fmt, clippy, test, ./test.sh)
6. Update Status → `Accepted` when complete
7. Update ROADMAP.md if applicable

**User's Actions:**
- Give explicit "start implementation" instruction
- Review implementation progress
- Verify quality gates passed

### Prohibited Actions (Claude)
- ❌ Proceed to next phase without user OK
- ❌ Say "If there are no issues, I'll proceed to the next phase" (explicit OK required)
- ❌ Write multiple phases at once
- ❌ Start implementation without explicit instruction

### ROADMAP Integration
- User specifies which ROADMAP feature to implement
- Update ROADMAP.md when creating PR

## How to Use This Process

### Starting a New Feature
```
User: "I want to implement feature X from ROADMAP.
       Please follow docs/adr/SPEC_DRIVEN_PROCESS.md"

Claude: [Reads SPEC_DRIVEN_PROCESS.md]
        [Creates ADR-XXX-feature-name.md with Requirements]
        [Asks for confirmation]
```

### Resuming In-Progress ADR
```
User: "Please continue with ADR-XXX.
       Follow docs/adr/SPEC_DRIVEN_PROCESS.md"

Claude: [Reads SPEC_DRIVEN_PROCESS.md]
        [Reads ADR-XXX-feature-name.md]
        [Checks current Status]
        [Continues from appropriate phase]
```

### Checking Current Status
Look at the ADR file's Status field:
- `Proposed: Requirements` → Waiting for requirements approval
- `Proposed: Design` → Waiting for design approval
- `Proposed: Plan` → Waiting for implementation plan approval
- `In Progress` → Implementation ongoing
- `Accepted` → Complete

## Template

Use `docs/adr/000-template-spec-driven.md` for new spec-driven ADRs.

Traditional ADRs (recording decisions after implementation) can still use `docs/adr/000-template.md`.

## Quality Requirements (rustile specific)

Every spec-driven ADR must address:

**Requirements Phase:**
- [ ] Zero-warning requirement
- [ ] Error handling strategy (anyhow::Result)
- [ ] Test strategy (unit/integration/edge)
- [ ] ADR-005 comment standard

**Design Phase:**
- [ ] Impact on 7-file module structure
- [ ] X11 event handling changes
- [ ] Logging strategy (error!/info!/debug!)
- [ ] BSP layout compatibility

**Implementation Plan Phase:**
- [ ] cargo fmt passes
- [ ] cargo clippy --all-targets --all-features -- -D warnings passes
- [ ] cargo test all tests pass
- [ ] ./test.sh manual verification

## Version History
- v1.0 (2025-10-01): Initial spec-driven process definition