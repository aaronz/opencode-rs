#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)/opencode-rust"
OUTPUT_DIR="${OUTPUT_DIR:-$PROJECT_DIR/test-reports}"
REPORT_FORMAT="${REPORT_FORMAT:-text}"
GENERATE_COVERAGE="${GENERATE_COVERAGE:-false}"
TEST_CATEGORY="${TEST_CATEGORY:-all}"
RUN_PACKAGE=""
TEST_TIMEOUT=""
SINGLE_THREADED="${SINGLE_THREADED:-false}"
SKIP_TESTS=""

export OUTPUT_DIR REPORT_FORMAT GENERATE_COVERAGE RUN_PACKAGE TEST_TIMEOUT

mkdir -p "$OUTPUT_DIR"

TESTS_PASSED=0
TESTS_FAILED=0
TESTS_IGNORED=0
TESTS_TOTAL=0

parse_cargo_test_output() {
    local input_file="$1"

    if [[ ! -f "$input_file" ]]; then
        echo "0 0 0 0"
        return
    fi

    local passed=0
    local failed=0
    local ignored=0
    local total=0

    local passed_count=$(grep -oE "[0-9]+ passed" "$input_file" 2>/dev/null | awk '{sum+=$1} END {print sum+0}')
    local failed_count=$(grep -oE "[0-9]+ failed" "$input_file" 2>/dev/null | awk '{sum+=$1} END {print sum+0}')
    local ignored_count=$(grep -oE "[0-9]+ ignored" "$input_file" 2>/dev/null | awk '{sum+=$1} END {print sum+0}')

    passed=${passed_count:-0}
    failed=${failed_count:-0}
    ignored=${ignored_count:-0}
    total=$((passed + failed + ignored))

    echo "$passed $failed $ignored $total"
}

build_cargo_args() {
    local args=()

    if [[ -n "$RUN_PACKAGE" ]]; then
        args+=("-p" "$RUN_PACKAGE")
    fi

    if [[ "$SINGLE_THREADED" == "true" ]]; then
        args+=("--test-threads=1")
    fi

    if [[ -n "$SKIP_TESTS" ]]; then
        for skip in $SKIP_TESTS; do
            args+=("--skip" "$skip")
        done
    fi

    echo "${args[@]}"
}

run_unit_tests() {
    echo "Running unit tests..."
    cd "$PROJECT_DIR"

    local output_file="$OUTPUT_DIR/unit-test-output.txt"
    local args=$(build_cargo_args)

    cargo test --lib $args -- --nocapture 2>&1 | tee "$output_file"

    echo "$output_file"
}

run_integration_tests() {
    echo "Running integration tests..."
    cd "$PROJECT_DIR"

    local output_file="$OUTPUT_DIR/integration-test-output.txt"
    local args=$(build_cargo_args)

    cargo test --test '*' $args -- --nocapture 2>&1 | tee "$output_file"

    echo "$output_file"
}

run_doc_tests() {
    echo "Running doc tests..."
    cd "$PROJECT_DIR"

    local output_file="$OUTPUT_DIR/doc-test-output.txt"
    local args=$(build_cargo_args)

    cargo test --doc $args -- --nocapture 2>&1 | tee "$output_file"

    echo "$output_file"
}

run_workspace_tests() {
    echo "Running workspace tests..."
    cd "$PROJECT_DIR"

    local output_file="$OUTPUT_DIR/workspace-test-output.txt"
    local args=$(build_cargo_args)

    cargo test --workspace $args -- --nocapture 2>&1 | tee "$output_file"

    echo "$output_file"
}

run_all_tests() {
    echo "Running all tests..."
    cd "$PROJECT_DIR"

    local output_file="$OUTPUT_DIR/test-output.txt"
    local args=$(build_cargo_args)

    cargo test $args -- --nocapture 2>&1 | tee "$output_file"

    echo "$output_file"
}

run_json_report() {
    local test_name="$1"
    echo "Running tests ($test_name) with JSON output..."
    cd "$PROJECT_DIR"

    local output_file="$OUTPUT_DIR/${test_name}-test-output.json"
    local args=$(build_cargo_args)

    cargo test $args -- --format=json 2>&1 | tee "$output_file"

    echo "$output_file"
}

generate_junit_xml() {
    local input_file="$1"
    local output_file="$2"

    if [[ ! -f "$input_file" ]]; then
        echo "Warning: Input file $input_file not found, skipping JUnit XML"
        return 1
    fi

    echo "Generating JUnit XML report..."

    local test_suite="opencode-rs"
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    read passed failed ignored total <<< "$(parse_cargo_test_output "$input_file")"

    local tests="${total:-0}"
    local failures="${failed:-0}"
    local errors=0
    local skipped="${ignored:-0}"
    local time=0

    cat > "$output_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="opencode-rs" tests="$tests" failures="$failures" errors="$errors" skipped="$skipped" time="$time" timestamp="$timestamp">
  <testsuite name="$test_suite" tests="$tests" failures="$failures" errors="$errors" skipped="$skipped" time="$time">
    <testcase name="all" classname="opencode-rs" time="$time">
EOF

    local failed_tests=$(grep "^    test " "$input_file" | grep " FAILED" | head -50)
    while IFS= read -r line || [[ -n "$line" ]]; do
        if [[ -z "$line" ]]; then continue; fi
        local test_name=$(echo "$line" | sed -n 's/.*test \([^ ]*\).*/\1/p')
        if [[ -n "$test_name" ]]; then
            echo "      <failure message=\"Test $test_name failed\" type=\"AssertionError\"/>" >> "$output_file"
        fi
    done <<< "$failed_tests"

    cat >> "$output_file" << EOF
    </testcase>
  </testsuite>
</testsuites>
EOF

    echo "JUnit XML written to $output_file"
}

generate_coverage_report() {
    if ! command -v cargo-llvm-cov &> /dev/null; then
        echo "Installing cargo-llvm-cov..."
        cargo install cargo-llvm-cov 2>/dev/null || {
            echo "Failed to install cargo-llvm-cov, skipping coverage"
            return 0
        }
    fi

    echo "Running tests with coverage..."
    cd "$PROJECT_DIR"

    local cov_cmd="cargo llvm-cov --no-cfg-coverage --fail-under-lines 0"
    if [[ -n "$RUN_PACKAGE" ]]; then
        $cov_cmd -p "$RUN_PACKAGE" --html --output-dir "$OUTPUT_DIR/coverage" 2>&1 | tee "$OUTPUT_DIR/coverage-output.txt"
    else
        $cov_cmd --all --html --output-dir "$OUTPUT_DIR/coverage" 2>&1 | tee "$OUTPUT_DIR/coverage-output.txt"
    fi

    echo "Coverage report generated in $OUTPUT_DIR/coverage/"
}

run_category_tests() {
    local start_time=$(date +%s)
    local output_file=""

    case "$TEST_CATEGORY" in
        unit)
            output_file=$(run_unit_tests)
            ;;
        integration)
            output_file=$(run_integration_tests)
            ;;
        doc)
            output_file=$(run_doc_tests)
            ;;
        workspace)
            output_file=$(run_workspace_tests)
            ;;
        all)
            output_file=$(run_all_tests)
            ;;
        *)
            echo "Unknown test category: $TEST_CATEGORY"
            echo "Valid categories: unit, integration, doc, workspace, all"
            return 1
            ;;
    esac

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    if [[ -f "$OUTPUT_DIR/test-output.txt" ]]; then
        read TESTS_PASSED TESTS_FAILED TESTS_IGNORED TESTS_TOTAL <<< "$(parse_cargo_test_output "$OUTPUT_DIR/test-output.txt")"
    elif [[ -n "$output_file" && -f "$output_file" ]]; then
        read TESTS_PASSED TESTS_FAILED TESTS_IGNORED TESTS_TOTAL <<< "$(parse_cargo_test_output "$output_file")"
    fi

    echo ""
    echo "========================================"
    echo "Test Results Summary"
    echo "========================================"
    echo "Passed:  ${TESTS_PASSED:-0}"
    echo "Failed:  ${TESTS_FAILED:-0}"
    echo "Ignored: ${TESTS_IGNORED:-0}"
    echo "Total:   ${TESTS_TOTAL:-0}"
    echo "Duration: ${duration}s"
    echo ""

    if [[ "$REPORT_FORMAT" == "junit" ]]; then
        generate_junit_xml "$output_file" "$OUTPUT_DIR/junit.xml"
    fi

    if [[ "$GENERATE_COVERAGE" == "true" ]]; then
        generate_coverage_report
    fi

    if [[ "${TESTS_FAILED:-0}" -gt 0 ]]; then
        echo "TESTS FAILED!"
        return 1
    else
        echo "ALL TESTS PASSED!"
        return 0
    fi
}

main() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  -h, --help           Show this help"
                echo "  --category CATEGORY Test category: unit, integration, doc, workspace, all (default: all)"
                echo "  --format FORMAT     Output format: text, json, junit (default: text)"
                echo "  --coverage          Generate coverage report"
                echo "  --package NAME      Run tests for specific package only"
                echo "  --timeout SECS      Test timeout in seconds (default: unlimited)"
                echo "  --single-threaded  Run tests single-threaded (for race-prone tests)"
                echo "  --skip TESTS        Skip tests matching pattern (can be repeated)"
                echo "  --output-dir DIR    Output directory (default: test-reports)"
                echo ""
                echo "Environment variables:"
                echo "  TEST_CATEGORY       Test category: unit, integration, doc, workspace, all"
                echo "  REPORT_FORMAT       Output format: text, json, junit"
                echo "  GENERATE_COVERAGE   Set to true to generate coverage"
                echo "  OUTPUT_DIR          Output directory"
                echo "  SINGLE_THREADED     Set to true for single-threaded execution"
                echo ""
                echo "Examples:"
                echo "  $0 --category unit                    # Run unit tests only"
                echo "  $0 --category integration --single-threaded  # Run integration tests single-threaded"
                echo "  $0 --category workspace --package opencode-config  # Run workspace tests for specific package"
                return 0
                ;;
            --category)
                TEST_CATEGORY="$2"
                shift 2
                ;;
            --format)
                REPORT_FORMAT="$2"
                shift 2
                ;;
            --coverage)
                GENERATE_COVERAGE="true"
                shift
                ;;
            --package)
                RUN_PACKAGE="$2"
                shift 2
                ;;
            --timeout)
                TEST_TIMEOUT="$2"
                shift 2
                ;;
            --single-threaded)
                SINGLE_THREADED="true"
                shift
                ;;
            --skip)
                if [[ -n "$SKIP_TESTS" ]]; then
                    SKIP_TESTS="$SKIP_TESTS|$2"
                else
                    SKIP_TESTS="$2"
                fi
                shift 2
                ;;
            --output-dir)
                OUTPUT_DIR="$2"
                mkdir -p "$OUTPUT_DIR"
                shift 2
                ;;
            *)
                echo "Unknown option: $1"
                echo "Use -h or --help for usage"
                return 1
                ;;
        esac
    done

    echo "========================================"
    echo "OpenCode RS Test Runner"
    echo "========================================"
    echo "Project: $PROJECT_DIR"
    echo "Output: $OUTPUT_DIR"
    echo "Format: $REPORT_FORMAT"
    echo "Category: $TEST_CATEGORY"
    echo "Coverage: $GENERATE_COVERAGE"
    echo ""
    echo "Starting test suite..."

    run_category_tests
}

main "$@"
