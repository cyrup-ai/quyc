//! Fluent API extensions and utilities
//!
//! Provides additional fluent interface components including download
//! functionality and progress tracking for enhanced user experience.

// Pure streams - no futures imports needed

use ystream::prelude::MessageChunk;



/// Main fluent builder for HTTP requests
///
/// Provides a fluent interface for building and executing HTTP requests
/// with method chaining and configuration options.
pub struct FluentBuilder {
    // Implementation will be added as needed
}

impl FluentBuilder {
    /// Create a new fluent builder
    #[must_use] 
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FluentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for download-specific operations
///
/// Provides a fluent interface for configuring and executing file downloads
/// with progress tracking and error handling.
pub struct DownloadBuilder {
    stream: crate::http::response::HttpDownloadStream,
}

impl DownloadBuilder {
    /// Create a new download builder with the provided stream
    ///
    /// # Arguments
    /// * `stream` - Download stream from HTTP client
    #[must_use] 
    pub fn new(stream: crate::http::response::HttpDownloadStream) -> Self {
        Self { stream }
    }

    /// Save the downloaded file to a local path
    ///
    /// Downloads the file from the stream and saves it to the specified
    /// local filesystem path with progress tracking.
    ///
    /// # Arguments
    /// * `local_path` - Local filesystem path where the file should be saved
    ///
    /// # Returns
    /// `DownloadProgress` - Download progress information
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3;
    ///
    /// let progress = Http3::new()
    ///     .download_file("https://example.com/large-file.zip")
    ///     .save("/tmp/downloaded-file.zip");
    ///
    /// println!("Downloaded {} bytes to {}",
    ///     progress.bytes_written,
    ///     progress.local_path);
    ///
    /// if let Some(percentage) = progress.progress_percentage() {
    ///     println!("Download completed: {:.1}%", percentage);
    /// }
    /// ```
    #[must_use] 
    pub fn save(self, local_path: &str) -> ystream::AsyncStream<DownloadProgress, 1024> {
        let local_path = local_path.to_string();
        let stream = self.stream;

        ystream::AsyncStream::with_channel(move |sender| {
            let mut total_written = 0;
            let mut chunk_count = 0;
            let mut total_size = None;

            // Use blocking file I/O - pure streams, no async
            let mut file = match std::fs::File::create(&local_path) {
                Ok(f) => f,
                Err(e) => {
                    let error_msg = format!("Failed to create file {local_path}: {e}");
                    ystream::emit!(sender, DownloadProgress::bad_chunk(error_msg));
                    return;
                }
            };

            for download_chunk in stream {
                match download_chunk {
                    crate::http::response::HttpDownloadChunk::Data { chunk, downloaded, total_size: chunk_total_size } => {
                        use std::io::Write;
                        let _bytes_written = match file.write(&chunk) {
                            Ok(n) => n,
                            Err(e) => {
                                let error_msg =
                                    format!("Failed to write to file {local_path}: {e}");
                                ystream::emit!(
                                    sender,
                                    DownloadProgress::bad_chunk(error_msg)
                                );
                                return;
                            }
                        };
                        total_written = downloaded;
                        if let Some(size) = chunk_total_size {
                            total_size = Some(size);
                        }
                        chunk_count += 1;
                    }
                    crate::http::response::HttpDownloadChunk::Complete => {
                        // Download completed successfully
                        break;
                    }
                    crate::http::response::HttpDownloadChunk::Error { message } => {
                        let error_msg = format!("Download error: {message}");
                        ystream::emit!(sender, DownloadProgress::bad_chunk(error_msg));
                        return;
                    }
                    crate::http::response::HttpDownloadChunk::Progress { .. } => {
                        // Progress-only updates, no data to write
                    }
                }
            }

            ystream::emit!(
                sender,
                DownloadProgress {
                    chunk_count,
                    bytes_written: total_written,
                    total_size,
                    local_path: local_path.clone(),
                    is_complete: true,
                    error_message: None,
                }
            );
        })
    }

    /// Set a custom destination path (alternative to save)
    ///
    /// Provides a fluent alternative to the save method for setting
    /// the download destination.
    ///
    /// # Arguments
    /// * `path` - Local filesystem path for the download
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3;
    ///
    /// let progress = Http3::new()
    ///     .download_file("https://example.com/file.zip")
    ///     .destination("/downloads/file.zip");
    /// ```
    #[must_use] 
    pub fn destination(self, path: &str) -> ystream::AsyncStream<DownloadProgress, 1024> {
        self.save(path)
    }

    /// Start the download with progress monitoring
    ///
    /// Alias for `save()` that emphasizes the streaming nature of the download.
    ///
    /// # Arguments
    /// * `local_path` - Local filesystem path for the download
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::Http3;
    ///
    /// let progress = Http3::new()
    ///     .download_file("https://example.com/file.zip")
    ///     .start("/downloads/file.zip");
    /// ```
    #[must_use] 
    pub fn start(self, local_path: &str) -> ystream::AsyncStream<DownloadProgress, 1024> {
        self.save(local_path)
    }
}

/// Download progress information for saved files
///
/// Contains detailed information about a completed or in-progress download
/// including byte counts, progress percentage, and completion status.
#[derive(Debug, Clone, Default)]
pub struct DownloadProgress {
    /// Number of chunks received during download
    pub chunk_count: u32,
    /// Total bytes written to local file
    pub bytes_written: u64,
    /// Total expected file size if known from headers
    pub total_size: Option<u64>,
    /// Local filesystem path where file was saved
    pub local_path: String,
    /// Whether the download completed successfully
    pub is_complete: bool,
    /// Error message if the download failed
    pub error_message: Option<String>,
}

impl ystream::prelude::MessageChunk for DownloadProgress {
    fn bad_chunk(error: String) -> Self {
        DownloadProgress {
            chunk_count: 0,
            bytes_written: 0,
            total_size: None,
            local_path: "[ERROR]".to_string(),
            is_complete: false,
            error_message: Some(error),
        }
    }

    fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    fn is_error(&self) -> bool {
        self.error_message.is_some()
    }
}

impl DownloadProgress {
    /// Calculate progress percentage if total size is known
    ///
    /// Returns the download progress as a percentage (0.0 to 100.0)
    /// if the total file size was provided in HTTP headers.
    ///
    /// # Returns
    /// `Option<f64>` - Progress percentage, or None if total size unknown
    ///
    /// # Examples
    /// ```no_run
    /// use quyc::builder::fluent::DownloadProgress;
    ///
    /// let progress = DownloadProgress {
    ///     chunk_count: 42,
    ///     bytes_written: 1024000,
    ///     total_size: Some(2048000),
    ///     local_path: "/tmp/file.zip".to_string(),
    ///     is_complete: false,
    /// };
    ///
    /// if let Some(percentage) = progress.progress_percentage() {
    ///     println!("Download progress: {:.1}%", percentage);
    /// }
    /// ```
    #[must_use] 
    pub fn progress_percentage(&self) -> Option<f64> {
        self.total_size.map(|total| {
            if total == 0 {
                100.0
            } else {
                // Use saturation arithmetic to avoid overflow and maintain precision
                // For maximum precision, convert to f64 early and avoid integer truncation
                // Precision loss acceptable for progress percentage calculations
                #[allow(clippy::cast_precision_loss)]
                let bytes_written = self.bytes_written as f64;
                #[allow(clippy::cast_precision_loss)]
                let total_size = total as f64;
                
                (bytes_written / total_size) * 100.0
            }
        })
    }

    /// Check if download is complete
    ///
    /// Returns true if the download has finished successfully.
    ///
    /// # Examples
    /// ```no_run
    /// # use quyc::builder::fluent::DownloadProgress;
    /// # let progress = DownloadProgress {
    /// #     chunk_count: 42,
    /// #     bytes_written: 2048000,
    /// #     total_size: Some(2048000),
    /// #     local_path: "/tmp/file.zip".to_string(),
    /// #     is_complete: true,
    /// # };
    /// if progress.is_finished() {
    ///     println!("Download completed: {}", progress.local_path);
    /// }
    /// ```
    #[must_use] 
    pub fn is_finished(&self) -> bool {
        self.is_complete
    }

    /// Get a human-readable status string
    ///
    /// Returns a formatted string describing the current download status.
    ///
    /// # Examples
    /// ```no_run
    /// # use quyc::builder::fluent::DownloadProgress;
    /// # let progress = DownloadProgress {
    /// #     chunk_count: 42,
    /// #     bytes_written: 1024000,
    /// #     total_size: Some(2048000),
    /// #     local_path: "/tmp/file.zip".to_string(),
    /// #     is_complete: false,
    /// # };
    /// println!("Status: {}", progress.status_string());
    /// ```
    #[must_use] 
    pub fn status_string(&self) -> String {
        if self.is_complete {
            format!(
                "Completed: {} bytes saved to {}",
                self.bytes_written, self.local_path
            )
        } else if let Some(percentage) = self.progress_percentage() {
            format!(
                "In progress: {:.1}% ({} / {} bytes)",
                percentage,
                self.bytes_written,
                self.total_size.unwrap_or(0)
            )
        } else {
            format!("In progress: {} bytes downloaded", self.bytes_written)
        }
    }
}
