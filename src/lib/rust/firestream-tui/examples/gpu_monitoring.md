# Example: Adding GPU Monitoring to the Node Details View

This example shows how to extend the TUI to add GPU monitoring visualization for nodes.

## Step 1: Extend the Backend Trait

Add a new method to get real-time GPU metrics in `src/backend/mod.rs`:

```rust
pub trait FirestreamBackend: Send + Sync {
    // ... existing methods ...
    
    // Add GPU metrics streaming
    fn stream_gpu_metrics(&self, node_id: &str) -> impl Stream<Item = ApiResult<GpuMetrics>> + Send;
}
```

## Step 2: Add GPU Metrics Model

Create a new model in `src/models/node.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub timestamp: DateTime<Utc>,
    pub devices: Vec<GpuDeviceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDeviceMetrics {
    pub index: u32,
    pub utilization: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: f64,
    pub power_draw: f64,
}
```

## Step 3: Update the Details View

Enhance `src/views/details_pane.rs` to show GPU metrics:

```rust
fn render_gpu_metrics(metrics: &[GpuDeviceMetrics], area: Rect, buf: &mut Buffer) {
    for (idx, gpu) in metrics.iter().enumerate() {
        let y = area.y + (idx * 4) as u16;
        
        // GPU header
        let header = Line::from(vec![
            Span::raw("GPU "),
            Span::raw(&gpu.index.to_string()),
            Span::raw(": "),
            Span::styled(
                format!("{:.0}%", gpu.utilization),
                Style::default().fg(utilization_color(gpu.utilization))
            ),
        ]);
        buf.set_line(area.x, y, &header, area.width);
        
        // Utilization bar
        let util_bar = create_progress_bar(gpu.utilization / 100.0, area.width - 2);
        buf.set_line(area.x + 1, y + 1, &util_bar, area.width - 2);
        
        // Memory usage
        let mem_text = format!(
            "Memory: {:.1}GB / {:.1}GB",
            gpu.memory_used as f64 / 1_073_741_824.0,
            gpu.memory_total as f64 / 1_073_741_824.0
        );
        buf.set_string(area.x + 1, y + 2, mem_text, Style::default());
        
        // Temperature and power
        let temp_power = format!(
            "Temp: {}°C | Power: {:.0}W",
            gpu.temperature, gpu.power_draw
        );
        buf.set_string(area.x + 1, y + 3, temp_power, Style::default());
    }
}

fn utilization_color(util: f64) -> Color {
    match util {
        u if u < 50.0 => Color::Green,
        u if u < 80.0 => Color::Yellow,
        _ => Color::Red,
    }
}

fn create_progress_bar(ratio: f64, width: u16) -> Line {
    let filled = (ratio * width as f64) as u16;
    let empty = width - filled;
    
    Line::from(vec![
        Span::styled("█".repeat(filled as usize), Style::default().fg(Color::Cyan)),
        Span::styled("░".repeat(empty as usize), Style::default().fg(Color::DarkGray)),
    ])
}
```

## Step 4: Add Event Handling

Update `src/app.rs` to handle GPU metrics updates:

```rust
impl App {
    async fn start_gpu_monitoring(&mut self, node_id: String) {
        let backend = self.backend.clone();
        let events = self.events.clone();
        
        tokio::spawn(async move {
            let mut stream = backend.stream_gpu_metrics(&node_id);
            
            while let Some(result) = stream.next().await {
                if let Ok(metrics) = result {
                    events.send(AppEvent::GpuMetricsUpdate(node_id.clone(), metrics));
                }
            }
        });
    }
    
    async fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            // ... existing events ...
            AppEvent::GpuMetricsUpdate(node_id, metrics) => {
                if let Some(ResourceDetails::Node(node)) = &mut self.selected_details {
                    if node.node.id == node_id {
                        // Update GPU metrics in the node details
                        node.gpu_metrics = Some(metrics);
                    }
                }
            }
        }
    }
}
```

## Step 5: Add Mock Data

Update `src/backend/mock_client.rs` to provide sample GPU metrics:

```rust
impl FirestreamBackend for MockClient {
    fn stream_gpu_metrics(&self, node_id: &str) -> impl Stream<Item = ApiResult<GpuMetrics>> {
        let interval = tokio::time::interval(Duration::from_secs(1));
        
        stream::unfold(interval, move |mut interval| async move {
            interval.tick().await;
            
            let metrics = GpuMetrics {
                timestamp: Utc::now(),
                devices: vec![
                    GpuDeviceMetrics {
                        index: 0,
                        utilization: rand::random::<f64>() * 100.0,
                        memory_used: 8_000_000_000 + (rand::random::<u64>() % 4_000_000_000),
                        memory_total: 16_000_000_000,
                        temperature: 60.0 + rand::random::<f64>() * 20.0,
                        power_draw: 200.0 + rand::random::<f64>() * 100.0,
                    }
                ],
            };
            
            Some((Ok(metrics), interval))
        })
    }
}
```

## Result

With these changes, when viewing a GPU-enabled node, the details pane will show:

```
worker-gpu-01
────────────────────────────────────────────────
Status:      Ready
Provider:    GCP
Type:        g2-standard-16
Spot:        No

Resources
────────────────────────────────────────────────
CPU:         16 cores (12.3 used)
Memory:      64 GB (48.2 GB used)

GPU Monitoring
────────────────────────────────────────────────
GPU 0: 87%
 ████████████████████░░░░
 Memory: 11.2GB / 16.0GB
 Temp: 75°C | Power: 285W
```

This example demonstrates:
- How to extend the backend trait
- Adding new data models
- Updating views to display new information
- Handling real-time updates
- Integrating with the mock backend

The same pattern can be used to add other features like:
- Kafka topic monitoring
- Build progress visualization
- Delta table statistics
- Custom resource editors
