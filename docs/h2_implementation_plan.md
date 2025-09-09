# H2 Implementation Plan - ystream Integration

## Core Discovery: recv_frame() is the Key Primitive

From h2 source analysis, the critical insight is that `Connection::recv_frame()` processes frame::Data directly:

```rust
// From proto/connection.rs line 517-520
Some(Data(frame)) => {
    tracing::trace!(?frame, "recv DATA");
    self.streams.recv_data(frame)?;  // Direct frame processing
}
```

## Implementation Strategy

### Phase 1: Create Sync Frame Processor
```rust
// New module: src/protocols/h2/frame_processor.rs
pub struct H2FrameProcessor {
    connection: h2::Connection<TcpStream>,
    waker: Arc<AtomicWaker>,
}

impl H2FrameProcessor {
    pub fn try_recv_frame(&mut self) -> Option<frame::Data> {
        // Use std::task::Context with no-op waker for sync polling
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        
        match Pin::new(&mut self.connection).poll(&mut cx) {
            Poll::Ready(Ok(())) => {
                // Connection processed frames, check for data
                self.extract_data_frames()
            }
            Poll::Pending => None,
            Poll::Ready(Err(_)) => None,
        }
    }
    
    fn extract_data_frames(&mut self) -> Option<frame::Data> {
        // Access internal frame buffer from connection
        // This requires either:
        // 1. Custom h2 fork with sync API
        // 2. Unsafe access to internal state
        // 3. Wrapper that captures frames during polling
    }
}
```

### Phase 2: Integrate with ystream
```rust
// Updated src/protocols/h2/connection.rs
impl H2Connection {
    pub fn send_request_stream(&self, request: Request<()>) -> AsyncStream<HttpChunk, 1024> {
        let processor = self.frame_processor.clone();
        
        AsyncStream::with_channel(move |sender| {
            // Background thread polls for frames
            loop {
                if let Some(data_frame) = processor.try_recv_frame() {
                    let bytes = data_frame.into_payload(); // Direct bytes access
                    let chunk = HttpChunk::new(bytes.to_vec());
                    emit!(sender, chunk);
                } else {
                    // No frames available, yield thread
                    std::thread::yield_now();
                }
            }
        })
    }
}
```

## Alternative Approach: Custom h2 Integration

Since h2 is heavily Future-based, we may need to create a custom integration:

### Option A: Fork h2 with Sync API
- Fork h2 crate to add sync frame processing methods
- Add `try_recv_frame()` and `try_send_frame()` methods
- Maintain compatibility with existing h2 API

### Option B: Frame Capture Wrapper
```rust
pub struct FrameCapture<T> {
    inner: h2::Connection<T>,
    captured_frames: crossbeam::queue::ArrayQueue<frame::Data>,
}

impl<T> FrameCapture<T> {
    fn poll_with_capture(&mut self, cx: &mut Context) -> Poll<Result<(), Error>> {
        // Poll inner connection and capture Data frames
        let result = Pin::new(&mut self.inner).poll(cx);
        
        // Extract any Data frames that were processed
        // This requires hooking into h2's internal frame processing
        
        result
    }
    
    pub fn try_recv_data(&self) -> Option<frame::Data> {
        self.captured_frames.pop()
    }
}
```

### Option C: Tokio Runtime Bridge (FORBIDDEN)
This approach would use tokio::spawn but is explicitly forbidden by user requirements.

## Recommended Implementation Path

1. **Create frame processor module** with sync polling capability
2. **Use noop_waker()** for Context creation in sync polling
3. **Extract frame::Data** using payload() method for raw bytes
4. **Integrate with AsyncStream::with_channel** pattern
5. **Use emit! macro** for ergonomic streaming

## Key Constraints
- ✅ NO async/await in streaming path
- ✅ NO Future-based APIs in public interface  
- ✅ Use only ystream patterns (with_channel, emit!)
- ✅ Direct frame::Data.payload() access for raw bytes
- ✅ Zero allocation where possible
- ❌ NO tokio::spawn or block_on
- ❌ NO simulation or placeholder code

## Next Steps
1. Implement H2FrameProcessor with sync polling
2. Create frame capture mechanism
3. Integrate with existing H2Connection methods
4. Test with real HTTP/2 streams
5. Remove all placeholder/simulation code