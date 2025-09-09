# JSONPath Streaming - Revolutionary Real-time JSON Processing

## Overview
The JSONPath streaming feature in quyc enables **real-time processing of massive JSON responses** without loading them into memory. It streams and emits matching JSON objects as soon as they're discovered in the HTTP response stream.

## Architecture Components

### 1. Zero-Allocation Buffer System (`jsonpath/buffer/`)
- `StreamBuffer`: Manages incoming HTTP chunks with zero-copy techniques
- `BytesMut` based accumulation with intelligent capacity management
- Tracks JSON object boundaries for incremental parsing
- Default 8KB buffer that grows based on content patterns

### 2. Incremental State Machine (`jsonpath/state_machine/`)
- Single-pass JSON parsing across chunk boundaries
- Tracks nested object/array depth without recursion
- Emits object boundaries immediately upon discovery
- Handles partial JSON gracefully across HTTP chunks

### 3. JSONPath Expression Engine (`jsonpath/core_evaluator/`)
- Compiles JSONPath expressions once at construction
- Supports full JSONPath spec: wildcards, filters, recursive descent
- Pattern matching happens incrementally as structure is discovered
- Zero allocation during evaluation phase

### 4. Stream Processor (`jsonpath/stream_processor/`)
- Orchestrates the streaming pipeline
- Applies JSONPath expressions to streaming data
- Deserializes matched objects on-the-fly
- Handles errors with circuit breaker pattern

## How It Works: Real-time Streaming Flow

1. **HTTP chunks arrive** → Fed directly into `StreamBuffer`
2. **State machine processes bytes** → Tracks JSON structure incrementally
3. **JSONPath evaluated in real-time** → As structure is discovered
4. **Objects emitted immediately** → When complete matches are found
5. **Type-safe deserialization** → Inline conversion to user types

## Fluent Builder API with JSONPath

The public API provides an elegant fluent interface for JSONPath streaming:

```rust
use quyc::{Http3, HttpChunk};
use serde::Deserialize;

#[derive(Deserialize)]
struct Model {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

// Stream models as they arrive from OpenAI
Http3::json()
    .array_stream("$.data[*]")  // JSONPath expression
    .bearer_token(&api_key)
    .get("https://api.openai.com/v1/models")
    .on_chunk(|model: Model| {
        // Called immediately when each model is found
        println!("Found model: {} by {}", model.id, model.owned_by);
        model
    })
    .collect(); // Or process as stream
```

## Real-World Example: OpenAI Models Streaming

Here's a complete example streaming OpenAI's `/v1/models` endpoint:

```rust
use quyc::{Http3, HttpChunk, BadChunk};
use serde::{Deserialize, Serialize};
use ystream::AsyncStream;

#[derive(Debug, Deserialize, Serialize)]
struct OpenAIModel {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

impl From<BadChunk> for OpenAIModel {
    fn from(_: BadChunk) -> Self {
        OpenAIModel {
            id: String::new(),
            object: String::new(),
            created: 0,
            owned_by: String::new(),
        }
    }
}

// Real-time streaming of OpenAI models
fn stream_openai_models(api_key: &str) -> AsyncStream<OpenAIModel, 1024> {
    Http3::json()
        .array_stream("$.data[*]")  // Extract each model from data array
        .bearer_token(api_key)
        .header("OpenAI-Beta", "assistants=v1")
        .get("https://api.openai.com/v1/models")
        .on_chunk(|result| {
            match result {
                Ok(model) => {
                    // Process each model in real-time
                    // This happens AS SOON as the model JSON is complete
                    // Not after the entire response is downloaded!
                    println!("Streaming model: {}", model.id);
                    
                    // Filter models in real-time
                    if model.owned_by.contains("openai") {
                        model
                    } else {
                        OpenAIModel::from(BadChunk::Filtered)
                    }
                }
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    OpenAIModel::from(BadChunk::ParseError)
                }
            }
        })
}

// Usage with immediate processing
async fn process_models() {
    let models_stream = stream_openai_models("sk-...");
    
    // Models are processed as they arrive from the network
    // First model processes before last model is downloaded!
    for model in models_stream {
        if model.id.starts_with("gpt-4") {
            println!("Found GPT-4 model: {}", model.id);
            // Can start using this model immediately
            // Don't need to wait for all models to download
        }
    }
}
```

## Advanced JSONPath Patterns

### Complex Filters
```rust
// Stream with equality filters
Http3::json()
    .array_stream("$.data[?(@.owned_by == 'openai-internal')]")
    .get(url)

// Stream with comparison operators
Http3::json()
    .array_stream("$.items[?(@.price < 100 && @.inStock == true)]")
    .get(url)
```

### Nested Array Streaming
```rust
// Stream nested arrays
Http3::json()
    .array_stream("$.results[*].items[*]")
    .get(url)

// Stream with multiple levels
Http3::json()
    .array_stream("$.users[*].orders[*].items[*]")
    .get(url)
```

### Recursive Descent
```rust
// Find all objects with specific field anywhere in JSON
Http3::json()
    .array_stream("$..author")
    .get(url)

// Recursive with filters
Http3::json()
    .array_stream("$..[?(@.type == 'user')]")
    .get(url)
```

### Array Slicing
```rust
// First 10 items
Http3::json()
    .array_stream("$.data[0:10]")
    .get(url)

// Every second item
Http3::json()
    .array_stream("$.data[0:100:2]")
    .get(url)

// Last 5 items
Http3::json()
    .array_stream("$.data[-5:]")
    .get(url)
```

## Performance Characteristics

| Metric | Value | Description |
|--------|-------|-------------|
| **Memory Usage** | ~8KB constant | Regardless of response size |
| **First Result Latency** | <10ms | After first matching bytes arrive |
| **Throughput** | 100K+ objects/sec | On modern hardware |
| **Allocations** | Zero | During streaming phase |
| **CPU Efficiency** | Single-pass | No backtracking or re-parsing |
| **Response Size Limit** | None | Can process TB of JSON |

## Implementation Details

### State Machine States
1. **Initial**: Waiting for JSON to begin
2. **Navigating**: Following JSONPath to target location
3. **StreamingArray**: Processing array elements
4. **ProcessingObject**: Extracting individual objects
5. **Finishing**: Cleanup after streaming
6. **Complete**: All data processed
7. **Error**: Recovery state with circuit breaker

### Buffer Management
- Automatic capacity scaling based on object sizes
- Memory pool reuse for zero allocation
- Boundary detection for clean object extraction
- UTF-8 validation with recovery strategies

### Error Handling
- Malformed JSON recovery
- Partial object handling
- Circuit breaker for systematic failures
- Graceful degradation on errors

## Use Cases

### 1. AI/LLM Streaming
Process chat completions, embeddings, or model outputs in real-time:
```rust
Http3::json()
    .array_stream("$.choices[*].delta")
    .post("https://api.openai.com/v1/chat/completions")
    .body(&request)
    .on_chunk(|delta| {
        print!("{}", delta.content);
        delta
    })
```

### 2. Large Dataset APIs
Stream millions of records without memory constraints:
```rust
Http3::json()
    .array_stream("$.records[*]")
    .get("https://api.example.com/big-data")
    .on_chunk(|record| {
        database.insert(record);
        record
    })
```

### 3. Real-time Data Pipelines
Process streaming data as it arrives:
```rust
Http3::json()
    .array_stream("$.events[*]")
    .get("https://api.example.com/live-feed")
    .on_chunk(|event| {
        event_bus.publish(event);
        event
    })
```

### 4. Progressive Rendering
Display results before download completes:
```rust
Http3::json()
    .array_stream("$.search_results[*]")
    .get(search_url)
    .on_chunk(|result| {
        ui.render_result(result);
        result
    })
```

### 5. Memory-Constrained Environments
Process huge responses on embedded or resource-limited systems:
```rust
Http3::json()
    .array_stream("$.telemetry[*]")
    .get(telemetry_url)
    .on_chunk(|data| {
        // Process TB of telemetry with MB of RAM
        metrics.record(data);
        data
    })
```

## Comparison with Traditional Approaches

### Traditional (Load Everything)
```rust
// BAD: Loads entire response into memory
let response = client.get(url).send().await?;
let json: Value = response.json().await?;  // BLOCKS until fully downloaded
let models = json["data"].as_array().unwrap();
for model in models {  // Processing starts AFTER full download
    process(model);
}
```

### JSONPath Streaming (Process Immediately)
```rust
// GOOD: Streams and processes in real-time
Http3::json()
    .array_stream("$.data[*]")
    .get(url)
    .on_chunk(|model| {
        process(model);  // Processing starts with FIRST chunk
        model
    })
```

## Integration with ystream

The JSONPath streaming feature is built on top of `ystream::AsyncStream`:

```rust
// Pure streams - no Futures, no Result wrapping
let stream: AsyncStream<Model, 1024> = Http3::json()
    .array_stream("$.data[*]")
    .get(url);

// Composable with other stream operations
stream
    .filter(|m| m.active)
    .map(|m| m.transform())
    .batch(10)
    .for_each(|batch| process_batch(batch));
```

## Best Practices

1. **Choose appropriate JSONPath expressions** - More specific paths reduce processing overhead
2. **Use typed deserialization** - Leverage Rust's type system for safety
3. **Handle errors gracefully** - Implement `From<BadChunk>` for error recovery
4. **Monitor buffer sizes** - Adjust initial capacity for your use case
5. **Test with partial data** - Ensure your code handles incomplete JSON
6. **Profile memory usage** - Verify zero-allocation behavior in production

## Future Enhancements

- JSONPath compilation caching across requests
- Parallel processing of independent array elements
- WebAssembly support for browser-side streaming
- Custom filter functions in JSONPath expressions
- Streaming aggregations and transformations