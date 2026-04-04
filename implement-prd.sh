#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_DIR="$(dirname "$SCRIPT_DIR")"
PRD_FILE="$WORKSPACE_DIR/../PRD.md"

MODEL="${1:-opencode/minimax-m2.5-free}"

if [ ! -f "$PRD_FILE" ]; then
    echo "Error: PRD.md not found at $PRD_FILE"
    exit 1
fi

PRD_CONTENT=$(cat "$PRD_FILE")

echo "========================================"
echo "Spec Kit workspace - PRD Implementation"
echo "方法论: constitution → specify → plan → tasks → implement"
echo "模型: $MODEL"
echo "========================================"
echo "PRD: $PRD_FILE"
echo ""

OUTPUT_DIR="$SCRIPT_DIR/outputs"
mkdir -p "$OUTPUT_DIR"

CONSTITUTION_FILE="$OUTPUT_DIR/constitution.md"
SPEC_FILE="$OUTPUT_DIR/spec.md"
PLAN_FILE="$OUTPUT_DIR/plan.md"
TASKS_FILE="$OUTPUT_DIR/tasks.md"

mkdir -p "$WORKSPACE_DIR/.specify/memory"
mkdir -p "$WORKSPACE_DIR/.specify/templates"
mkdir -p "$WORKSPACE_DIR/.specify/specs"

if [ ! -f "$WORKSPACE_DIR/.specify/memory/constitution.md" ]; then
    if [ -f "$WORKSPACE_DIR/.specify/templates/constitution-template.md" ]; then
        cp "$WORKSPACE_DIR/.specify/templates/constitution-template.md" "$WORKSPACE_DIR/.specify/memory/constitution.md"
    fi
fi

echo "[Step 1/5] Constitution - 建立项目原则..."
CONSTITUTION_PROMPT="You are creating a project constitution.

## Task
Update the project constitution at \`.specify/memory/constitution.md\`. This file is a TEMPLATE containing placeholder tokens in square brackets (e.g. \`[PROJECT_NAME]\`, \`[PRINCIPLE_1_NAME]\`). Your job is to (a) collect/derive concrete values, (b) fill the template precisely, and (c) propagate any amendments across dependent artifacts.

## Requirements Document
$PRD_CONTENT

## Execution Steps
1. Load the existing constitution at \`.specify/memory/constitution.md\`
2. Analyze the requirements document to understand the project
3. Fill all placeholder tokens with concrete values derived from the requirements
4. Define principles appropriate for this specific project based on what the requirements demand
5. Ensure each Principle section has: succinct name, non-negotiable rules, explicit rationale
6. Write the completed constitution to \`.specify/memory/constitution.md\`

## Output
Save the final constitution to: $CONSTITUTION_FILE"

opencode run -m "$MODEL" "$CONSTITUTION_PROMPT"

echo ""
echo "[Step 2/5] Specify - 定义需求规范..."
SPECIFY_PROMPT="You are creating a feature specification.

## Task
Create a detailed specification based on the requirements document, focusing on WHAT users need and WHY (not HOW to implement).

## Requirements Document
$PRD_CONTENT

## Constitution
$(cat "$WORKSPACE_DIR/.specify/memory/constitution.md" 2>/dev/null || echo "Not yet created")

## Execution Steps
1. Generate a concise short name for the feature based on the requirements
2. Parse the requirements to extract key concepts: actors, actions, data, constraints
3. Create User Scenarios & Testing section
4. Generate Functional Requirements (each must be testable)
5. Define Success Criteria (measurable, technology-agnostic)
6. Identify Key Entities involved

## Output
Save the specification to: $SPEC_FILE"

opencode run -m "$MODEL" "$SPECIFY_PROMPT"

echo ""
echo "[Step 3/5] Plan - 创建技术实现计划..."
PLAN_PROMPT="You are creating a technical implementation plan.

## Task
Create an implementation plan following the plan template structure.

## Specification
$(cat "$SPEC_FILE" 2>/dev/null || echo "Specification not yet created")

## Constitution
$(cat "$WORKSPACE_DIR/.specify/memory/constitution.md" 2>/dev/null || echo "Constitution not yet created")

## Plan Content Required
1. Technical Context (infer appropriate tech stack from requirements)
2. Constitution Check (verify alignment with principles)
3. Phase 0: Research (resolve unknowns)
4. Phase 1: Design (data model, contracts, quickstart)
5. Phase 2: Implementation breakdown
6. File structure and module organization

## Output
Save the plan to: $PLAN_FILE"

opencode run -m "$MODEL" "$PLAN_PROMPT"

echo ""
echo "[Step 4/5] Tasks - 生成任务清单..."
TASKS_PROMPT="You are generating an actionable task list.

## Task
Create a detailed, dependency-ordered task list from the implementation plan.

## Implementation Plan
$(cat "$PLAN_FILE" 2>/dev/null || echo "Plan not yet created")

## Task Generation Rules
1. Organize by user story to enable independent implementation and testing
2. Use strict checklist format: \`- [ ] [TaskID] [P?] [Story?] Description with file path\`
3. Task IDs: Sequential (T001, T002, T003...)
4. [P] marker: Include ONLY if task is parallelizable (different files, no dependencies)
5. [Story] label: Format [US1], [US2], etc. for user story phase tasks

## Phase Structure
- Phase 1: Setup (project initialization)
- Phase 2: Foundational (blocking prerequisites)
- Phase 3+: User Stories in priority order
- Final Phase: Polish & Cross-Cutting Concerns

## Output
Save the task list to: $TASKS_FILE"

opencode run -m "$MODEL" "$TASKS_PROMPT"

echo ""
echo "[Step 5/5] Implement - 执行实现..."
IMPLEMENT_PROMPT="You are implementing a project based on the task list.

## Task
Execute all tasks from the task list to build the complete application.

## Task List
$(cat "$TASKS_FILE" 2>/dev/null || echo "Tasks not yet created")

## Implementation Plan
$(cat "$PLAN_FILE" 2>/dev/null || echo "Plan not yet created")

## Execution Rules
1. Complete each phase before moving to the next
2. Respect dependencies - sequential tasks in order, parallel tasks [P] can run together
3. Mark completed tasks as [X] in the tasks file
4. Report progress after each completed task
5. Halt execution if any non-parallel task fails

## Output
Implement all code files according to the task list. Create the complete working application."

opencode run -m "$MODEL" "$IMPLEMENT_PROMPT"

echo ""
echo "========================================"
echo "Spec Kit workspace 实现完成!"
echo "========================================"
echo ""
echo "Output files:"
echo "  - Constitution: $CONSTITUTION_FILE"
echo "  - Specification: $SPEC_FILE"
echo "  - Implementation Plan: $PLAN_FILE"
echo "  - Task List: $TASKS_FILE"
