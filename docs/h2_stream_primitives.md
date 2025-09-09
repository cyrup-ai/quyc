# H2 Stream Primitives Research

## Key Finding: h2::Connection::poll() is the Core Primitive

From h2 source code analysis:

### Connection Polling Mechanism
```rust
// From client.rs line 1438-1451
impl Future for Connection<T, B> {
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.maybe_close_connection_if_no_streams();
        let result = self.inner.poll(cx).map_err(Into::into);
        // Connection polling drives all frame processing
    }
}
```

### Frame Processing Pipeline
```rust
// From proto/connection.rs line 350-351
fn recv_frame(&mut self, frame: Option<Frame>) -> Result<ReceivedFrame, Error> {
    match frame {
        Some(Data(frame)) => {
            // Direct frame::Data processing - NO Futures
            self.streams.recv_data(frame)?;
        }
        // Other frame types...
    }
}
```

### Direct Frame::Data Access
```rust
// From frame/data.rs lines 78-100
impl Data<Bytes> {
    pub fn payload(&self) -> &Bytes {  // Direct bytes access
        &self.data
    }
    
    pub fn into_payload(self) -> Bytes {  // Consume frame for bytes
        self.data
    }
}
```

### Stream Data Processing
```rust
// From proto/streams/recv.rs line 579
pub fn recv_data(&mut self, frame: frame::Data, stream: &mut store::Ptr) -> Result<(), Error> {
    // Direct frame processing - extracts bytes from frame.payload()
    let bytes = frame.into_payload();  // Get raw bytes
    // Process bytes directly without Futures
}
```

## Allowed ystream Patterns

### Pattern 1: AsyncStream::with_channel
```rust
AsyncStream::<ChunkType, 1024>::with_channel(move |sender| {
    // Producer thread - NO async/await
    for data in data_source {
        emit!(sender, chunk);
    }
});
```

### Pattern 2: spawn_stream  
```rust
spawn_stream(move |sender| {
    // Background streaming work - NO async/await
    loop {
        let chunk = process_data();
        emit!(sender, chunk);
    }
});
```

### Pattern 3: spawn_task
```rust
spawn_task(|| {
    // Single computation - NO async/await
    compute_result()
});
```

## FORBIDDEN Patterns
- ❌ async/await
- ❌ Future::poll() 
- ❌ tokio::spawn()
- ❌ block_on()
- ❌ Any Future-based APIs

## Implementation Strategy

### Core Approach
1. Use h2::Connection::poll() in background thread
2. Extract frame::Data from polling results  
3. Use frame.payload() or frame.into_payload() for raw bytes
4. Emit bytes using with_channel + emit! pattern

### Concrete Implementation Pattern
```rust
AsyncStream::<HttpChunk, 1024>::with_channel(move |sender| {
    // Background thread polls h2::Connection
    loop {
        // Poll connection for frames (using std polling, not async)
        match connection.poll_frames() {  // Hypothetical sync API
            Some(frame::Data(data_frame)) => {
                let bytes = data_frame.into_payload();
                let chunk = HttpChunk::new(bytes.to_vec());
                emit!(sender, chunk);
            }
            Some(_other_frame) => continue,
            None => break,
        }
    }
});
```

## Key Insight
h2 provides frame::Data with direct bytes access via payload() method. The challenge is accessing this without Future-based polling. Need to find or create sync polling mechanism for h2::Connection.