# Spec-Driven ADR Guidelines

Plan features incrementally through ADRs with explicit user approval at each phase.

## Process

```
Requirements → Design → Implementation → Verification
```

Each phase requires **explicit user OK** before proceeding.

## Phase Definitions

### 1. Requirements
**Output**: What problem to solve, functional/non-functional requirements, acceptance criteria
**Approval**: User reviews and says "OK" or requests changes
**Status**: `Proposed: Requirements`

### 2. Design
**Output**: How to solve it (approach, key changes, testing strategy)
**Approval**: User reviews and says "OK" or requests changes
**Status**: `Proposed: Design`

### 3. Implementation
**Trigger**: User gives explicit "start implementation" instruction
**Output**: Working code with iterative feedback during development
**Status**: `In Progress`
**Note**: User can provide feedback during implementation; adjustments are expected

### 4. Verification
**Output**: Test results and verification evidence documented in ADR
**Approval**: User reviews evidence and says "OK" to accept
**Status**: `Accepted`
**Note**: Claude automatically fills Verification section after implementation, then waits for user approval

## Core Rules

**Sequential progression**: Complete one phase before starting the next

**Explicit approval**: Claude must ask and wait for user OK at each phase boundary

**No skipping**: All phases required, no combining multiple phases

**Evidence required**: Verification section must be filled before marking Accepted

## Usage

**Start new feature**:
```
User: "Implement X. Follow SPEC_DRIVEN_GUIDELINES.md"
Claude: [Creates ADR, fills Requirements, waits for OK]
```

**Resume in-progress**:
```
User: "Continue ADR-XXX. Follow SPEC_DRIVEN_GUIDELINES.md"
Claude: [Reads ADR, checks Status, continues from current phase]
```

## Template

Use `000-template-spec-driven.md` for new spec-driven ADRs.
