#!/bin/bash
#
# Snapshot Generation Script for opencode-rs
# Fetches models.dev API and generates Rust-embeddable snapshot data
#
# Usage: ./scripts/generate-snapshot.sh [--check]
#   --check: Only check if update is needed, don't write
#
# Environment variables:
#   MODELS_DEV_URL: Override the API URL (default: https://models.dev/api.json)
#   SNAPSHOT_OUTPUT: Override output file path
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../opencode-rust" && pwd)"
CATALOG_DIR="$PROJECT_DIR/crates/llm/src/catalog"

# Default values
MODELS_DEV_URL="${MODELS_DEV_URL:-https://models.dev/api.json}"
SNAPSHOT_OUTPUT="${SNAPSHOT_OUTPUT:-$CATALOG_DIR/snapshot_catalog.json}"
SNAPSHOT_VERSION="${SNAPSHOT_VERSION:-1}"

# Timestamp in ISO 8601 format
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

# Check mode flag
CHECK_MODE=false
if [[ "$1" == "--check" ]]; then
    CHECK_MODE=true
fi

echo "========================================"
echo "Snapshot Generation Script"
echo "========================================"
echo "API URL: $MODELS_DEV_URL"
echo "Output: $SNAPSHOT_OUTPUT"
echo "Generated at: $GENERATED_AT"
echo "Snapshot version: $SNAPSHOT_VERSION"
echo ""

# Check for required tools
command -v curl >/dev/null 2>&1 || { echo "Error: curl is required but not installed." >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "Error: python3 is required but not installed." >&2; exit 1; }

# Create temp directory for processing
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "Fetching models.dev API data..."
API_RESPONSE="$TEMP_DIR/api_response.json"
HTTP_CODE=$(curl -s -w "%{http_code}" -o "$API_RESPONSE" "$MODELS_DEV_URL" --max-time 30 --retry 3)

if [[ "$HTTP_CODE" != "200" ]]; then
    echo "Error: Failed to fetch API (HTTP $HTTP_CODE)" >&2
    exit 1
fi

echo "Transforming data to SnapshotCatalog format..."

cat > "$TEMP_DIR/transform.py" << 'PYTHON_SCRIPT'
import sys
import json
import os
from datetime import datetime

# Read API response
api_response_path = os.environ.get('API_RESPONSE', '/dev/stdin')
with open(api_response_path, 'r') as f:
    api_data = json.load(f)

# Configuration
snapshot_version = os.environ.get('SNAPSHOT_VERSION', '1')
generated_at = os.environ.get('GENERATED_AT', datetime.utcnow().isoformat() + 'Z')

# Status mapping
status_map = {
    'active': 'Active',
    'beta': 'Beta',
    'alpha': 'Alpha',
    'deprecated': 'Deprecated'
}

def transform_modalities(modalities):
    """Transform modalities to string arrays."""
    if not modalities:
        return {'input': [], 'output': []}
    return {
        'input': modalities.get('input', []),
        'output': modalities.get('output', [])
    }

def transform_cost(cost):
    """Transform cost info."""
    if not cost:
        return {'input': 0.0, 'output': 0.0, 'cache_read': 0.0, 'cache_write': 0.0}
    return {
        'input': cost.get('input', 0.0) or 0.0,
        'output': cost.get('output', 0.0) or 0.0,
        'cache_read': cost.get('cache_read', 0.0) or 0.0,
        'cache_write': cost.get('cache_write', 0.0) or 0.0
    }

def transform_limits(limit):
    """Transform limit info."""
    if not limit:
        return {'context': 0, 'input': None, 'output': 0}
    return {
        'context': limit.get('context', 0) or 0,
        'input': limit.get('input'),
        'output': limit.get('output', 0) or 0
    }

def transform_capabilities(model):
    """Transform model capabilities."""
    modalities = model.get('modalities')
    interleaved = model.get('interleaved')

    return {
        'attachment': model.get('attachment', False),
        'reasoning': model.get('reasoning', False),
        'tool_call': model.get('tool_call', False),
        'temperature': model.get('temperature', False) or False,
        'structured_output': False,
        'interleaved': interleaved is not None,
        'open_weights': model.get('open_weights', False),
        'input_modalities': transform_modalities(modalities).get('input', []),
        'output_modalities': transform_modalities(modalities).get('output', [])
    }

def transform_model(model_id, model, provider_id):
    """Transform a single model."""
    status_str = model.get('status', 'active')
    status = status_map.get(status_str.lower(), 'Active')

    return {
        'id': model_id,
        'display_name': model.get('name', model_id),
        'family': model.get('family'),
        'provider_id': provider_id,
        'capabilities': transform_capabilities(model),
        'cost': transform_cost(model.get('cost')),
        'limits': transform_limits(model.get('limit')),
        'status': status
    }

def transform_provider(provider_id, provider):
    """Transform a single provider."""
    return {
        'id': provider_id,
        'display_name': provider.get('name', provider_id),
        'api_base_url': provider.get('api'),
        'docs_url': provider.get('doc'),
        'env_vars': provider.get('env', []),
        'npm_package': provider.get('npm'),
        'source': 'ModelsDev',
        'models': {
            model_id: transform_model(model_id, model, provider_id)
            for model_id, model in provider.get('models', {}).items()
        }
    }

# Build snapshot catalog
snapshot = {
    'snapshot_version': snapshot_version,
    'generated_at': generated_at,
    'providers': {
        provider_id: transform_provider(provider_id, provider)
        for provider_id, provider in api_data.items()
    }
}

# Output
output_path = os.environ.get('SNAPSHOT_OUTPUT')
if output_path:
    with open(output_path, 'w') as f:
        json.dump(snapshot, f, indent=2)
    print(f"Written: {output_path}")
else:
    json.dump(snapshot, sys.stdout, indent=2)

# Summary
provider_count = len(snapshot['providers'])
model_count = sum(len(p['models']) for p in snapshot['providers'].values())
print(f"Providers: {provider_count}, Models: {model_count}")
PYTHON_SCRIPT

export API_RESPONSE
export SNAPSHOT_OUTPUT
export SNAPSHOT_VERSION
export GENERATED_AT

python3 "$TEMP_DIR/transform.py"

echo ""
echo "========================================"
echo "Snapshot generation complete!"
echo "========================================"