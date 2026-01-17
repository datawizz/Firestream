//! Docker Manager CLI
//!
//! A comprehensive command-line tool for managing Docker resources.

use anyhow::Result;
use clap::{Parser, Subcommand};
use docker_manager::{DockerManager, utils};
use comfy_table::{Table, Cell};
use tracing::error;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "docker-manager")]
#[command(about = "A comprehensive Docker management tool", long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Container management commands
    Container {
        #[command(subcommand)]
        command: ContainerCommands,
    },
    
    /// Image management commands
    Image {
        #[command(subcommand)]
        command: ImageCommands,
    },
    
    /// Volume management commands
    Volume {
        #[command(subcommand)]
        command: VolumeCommands,
    },
    
    /// Network management commands
    Network {
        #[command(subcommand)]
        command: NetworkCommands,
    },
    
    /// System management commands
    System {
        #[command(subcommand)]
        command: SystemCommands,
    },
    
    /// Docker Compose commands
    Compose {
        #[command(subcommand)]
        command: ComposeCommands,
    },
    
    /// Build Docker images
    Build {
        /// Build context path
        #[arg(default_value = ".")]
        context: String,
        
        /// Image tag
        #[arg(short, long)]
        tag: String,
        
        /// Dockerfile path
        #[arg(short = 'f', long)]
        dockerfile: Option<String>,
        
        /// Build arguments
        #[arg(long = "build-arg")]
        build_args: Vec<String>,
        
        /// Don't use cache
        #[arg(long)]
        no_cache: bool,
    },
}

#[derive(Subcommand)]
enum ContainerCommands {
    /// List containers
    List {
        /// Show all containers (not just running)
        #[arg(short, long)]
        all: bool,
    },
    
    /// Start a container
    Start {
        /// Container name or ID
        container: String,
    },
    
    /// Stop a container
    Stop {
        /// Container name or ID
        container: String,
        
        /// Timeout in seconds
        #[arg(short, long)]
        timeout: Option<i64>,
    },
    
    /// Restart a container
    Restart {
        /// Container name or ID
        container: String,
        
        /// Timeout in seconds
        #[arg(short, long)]
        timeout: Option<i64>,
    },
    
    /// Remove a container
    Remove {
        /// Container name or ID
        container: String,
        
        /// Force removal
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show container logs
    Logs {
        /// Container name or ID
        container: String,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        
        /// Number of lines to show from the end
        #[arg(long)]
        tail: Option<String>,
    },
    
    /// Execute command in container
    Exec {
        /// Container name or ID
        container: String,
        
        /// Command to execute
        #[arg(required = true)]
        command: Vec<String>,
    },
    
    /// Show container statistics
    Stats {
        /// Container name or ID
        container: String,
    },
    
    /// Create a new container
    Create {
        /// Container name
        #[arg(short, long)]
        name: String,
        
        /// Image to use
        #[arg(short, long)]
        image: String,
    },
}

#[derive(Subcommand)]
enum ImageCommands {
    /// List images
    List {
        /// Show all images
        #[arg(short, long)]
        all: bool,
    },
    
    /// Pull an image
    Pull {
        /// Image name
        image: String,
        
        /// Image tag
        #[arg(short, long, default_value = "latest")]
        tag: String,
    },
    
    /// Remove an image
    Remove {
        /// Image name or ID
        image: String,
        
        /// Force removal
        #[arg(short, long)]
        force: bool,
    },
    
    /// Tag an image
    Tag {
        /// Source image
        source: String,
        
        /// Target repository
        target: String,
        
        /// Tag name
        #[arg(default_value = "latest")]
        tag: String,
    },
    
    /// Search for images
    Search {
        /// Search term
        term: String,
        
        /// Limit results
        #[arg(short, long)]
        limit: Option<u64>,
    },
    
    /// Show image history
    History {
        /// Image name or ID
        image: String,
    },
    
    /// Prune unused images
    Prune {
        /// Remove all unused images, not just dangling ones
        #[arg(short, long)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum VolumeCommands {
    /// List volumes
    List,
    
    /// Create a volume
    Create {
        /// Volume name
        name: String,
        
        /// Volume driver
        #[arg(short, long, default_value = "local")]
        driver: String,
    },
    
    /// Remove a volume
    Remove {
        /// Volume name
        volume: String,
        
        /// Force removal
        #[arg(short, long)]
        force: bool,
    },
    
    /// Inspect a volume
    Inspect {
        /// Volume name
        volume: String,
    },
    
    /// Prune unused volumes
    Prune,
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// List networks
    List,
    
    /// Create a network
    Create {
        /// Network name
        name: String,
        
        /// Network driver
        #[arg(short, long, default_value = "bridge")]
        driver: String,
        
        /// Internal network
        #[arg(long)]
        internal: bool,
        
        /// Attachable network
        #[arg(long)]
        attachable: bool,
    },
    
    /// Remove a network
    Remove {
        /// Network name or ID
        network: String,
    },
    
    /// Connect container to network
    Connect {
        /// Container name or ID
        container: String,
        
        /// Network name or ID
        network: String,
    },
    
    /// Disconnect container from network
    Disconnect {
        /// Container name or ID
        container: String,
        
        /// Network name or ID
        network: String,
        
        /// Force disconnection
        #[arg(short, long)]
        force: bool,
    },
    
    /// Inspect a network
    Inspect {
        /// Network name or ID
        network: String,
    },
    
    /// Prune unused networks
    Prune,
}

#[derive(Subcommand)]
enum SystemCommands {
    /// Show Docker version
    Version,
    
    /// Show Docker system information
    Info,
    
    /// Show disk usage
    Df,
    
    /// Prune all unused resources
    Prune,
    
    /// Check system health
    Health,
}

#[derive(Subcommand)]
enum ComposeCommands {
    /// Start services
    Up {
        /// Working directory with docker-compose.yml
        #[arg(short, long, default_value = ".")]
        dir: String,
        
        /// Run in detached mode
        #[arg(short, long)]
        detach: bool,
        
        /// Services to start
        services: Vec<String>,
    },
    
    /// Stop services
    Down {
        /// Working directory with docker-compose.yml
        #[arg(short, long, default_value = ".")]
        dir: String,
        
        /// Remove volumes
        #[arg(short, long)]
        volumes: bool,
        
        /// Remove orphan containers
        #[arg(long)]
        remove_orphans: bool,
    },
    
    /// Show service logs
    Logs {
        /// Working directory with docker-compose.yml
        #[arg(short, long, default_value = ".")]
        dir: String,
        
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
        
        /// Number of lines to show
        #[arg(long)]
        tail: Option<String>,
        
        /// Services to show logs for
        services: Vec<String>,
    },
    
    /// List services
    Ps {
        /// Working directory with docker-compose.yml
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
    
    /// Restart services
    Restart {
        /// Working directory with docker-compose.yml
        #[arg(short, long, default_value = ".")]
        dir: String,
        
        /// Services to restart
        services: Vec<String>,
    },
    
    /// Build services
    Build {
        /// Working directory with docker-compose.yml
        #[arg(short, long, default_value = ".")]
        dir: String,
        
        /// Don't use cache
        #[arg(long)]
        no_cache: bool,
        
        /// Services to build
        services: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    let filter = if cli.verbose {
        EnvFilter::new("docker_manager=debug,info")
    } else {
        EnvFilter::new("docker_manager=info,warn")
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    // Create Docker manager
    let docker = match DockerManager::new().await {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to connect to Docker: {}", e);
            std::process::exit(1);
        }
    };
    
    // Execute command
    match cli.command {
        Commands::Container { command } => handle_container_command(docker, command).await?,
        Commands::Image { command } => handle_image_command(docker, command).await?,
        Commands::Volume { command } => handle_volume_command(docker, command).await?,
        Commands::Network { command } => handle_network_command(docker, command).await?,
        Commands::System { command } => handle_system_command(docker, command).await?,
        Commands::Compose { command } => handle_compose_command(docker, command).await?,
        Commands::Build { context, tag, dockerfile, build_args, no_cache } => {
            handle_build_command(docker, context, tag, dockerfile, build_args, no_cache).await?
        }
    }
    
    Ok(())
}

async fn handle_container_command(docker: DockerManager, command: ContainerCommands) -> Result<()> {
    match command {
        ContainerCommands::List { all } => {
            let containers = docker.list_containers(all).await?;
            
            let mut table = Table::new();
            table.set_header(vec!["ID", "Name", "Image", "Status", "Ports"]);
            
            for container in containers {
                let id = container.id.as_deref().unwrap_or("").chars().take(12).collect::<String>();
                let names = container.names.as_ref()
                    .map(|n| n.join(", ").replace('/', ""))
                    .unwrap_or_default();
                let image = container.image.as_deref().unwrap_or("");
                let status = container.status.as_deref().unwrap_or("");
                let ports = container.ports.as_ref()
                    .map(|p| utils::format_ports(p))
                    .unwrap_or_default();
                
                table.add_row(vec![
                    Cell::new(id),
                    Cell::new(names),
                    Cell::new(image),
                    Cell::new(status),
                    Cell::new(ports),
                ]);
            }
            
            println!("{}", table);
        }
        
        ContainerCommands::Start { container } => {
            docker.start_container(&container).await?;
            println!("Container '{}' started", container);
        }
        
        ContainerCommands::Stop { container, timeout } => {
            docker.stop_container(&container, timeout).await?;
            println!("Container '{}' stopped", container);
        }
        
        ContainerCommands::Restart { container, timeout } => {
            docker.restart_container(&container, timeout).await?;
            println!("Container '{}' restarted", container);
        }
        
        ContainerCommands::Remove { container, force } => {
            docker.remove_container(&container, force).await?;
            println!("Container '{}' removed", container);
        }
        
        ContainerCommands::Logs { container, follow, tail } => {
            let logs = docker.container_logs(&container, follow, tail.as_deref()).await?;
            for log in logs {
                println!("{}", log);
            }
        }
        
        ContainerCommands::Exec { container, command } => {
            let cmd: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
            let output = docker.exec_in_container(&container, cmd, true).await?;
            print!("{}", output);
        }
        
        ContainerCommands::Stats { container } => {
            let stats = docker.container_stats(&container).await?;
            
            let cpu_percent = utils::calculate_cpu_percentage(&stats);
            let (mem_usage, mem_percent) = utils::calculate_memory_usage(&stats);
            
            println!("Container: {}", container);
            println!("CPU:       {:.2}%", cpu_percent);
            println!("Memory:    {} ({:.2}%)", utils::format_bytes(mem_usage as i64), mem_percent);
        }
        
        ContainerCommands::Create { name, image } => {
            let id = docker.create_container(&name, &image, None).await?;
            println!("Container '{}' created with ID: {}", name, id);
        }
    }
    
    Ok(())
}

async fn handle_image_command(docker: DockerManager, command: ImageCommands) -> Result<()> {
    match command {
        ImageCommands::List { all } => {
            let images = docker.list_images(all).await?;
            
            let mut table = Table::new();
            table.set_header(vec!["Repository", "Tag", "ID", "Created", "Size"]);
            
            for image in images {
                let id = image.id.chars().skip(7).take(12).collect::<String>();
                let size = utils::format_bytes(image.size);
                let created = chrono::DateTime::from_timestamp(image.created, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();
                
                for tag in &image.repo_tags {
                    let (repo, tag) = utils::parse_image_name(tag);
                    table.add_row(vec![
                        Cell::new(repo),
                        Cell::new(tag),
                        Cell::new(&id),
                        Cell::new(&created),
                        Cell::new(&size),
                    ]);
                }
            }
            
            println!("{}", table);
        }
        
        ImageCommands::Pull { image, tag } => {
            docker.pull_image(&image, Some(&tag)).await?;
            println!("Image '{}:{}' pulled successfully", image, tag);
        }
        
        ImageCommands::Remove { image, force } => {
            let removed = docker.remove_image(&image, force).await?;
            println!("Removed {} layers", removed.len());
        }
        
        ImageCommands::Tag { source, target, tag } => {
            docker.tag_image(&source, &target, &tag).await?;
            println!("Tagged '{}' as '{}:{}'", source, target, tag);
        }
        
        ImageCommands::Search { term, limit } => {
            let results = docker.search_images(&term, limit).await?;
            
            let mut table = Table::new();
            table.set_header(vec!["Name", "Description", "Stars", "Official"]);
            
            for result in results {
                let name = result.name.unwrap_or_default();
                let description = result.description.unwrap_or_default();
                let stars = result.star_count.unwrap_or(0).to_string();
                let official = if result.is_official.unwrap_or(false) { "Yes" } else { "No" };
                
                table.add_row(vec![
                    Cell::new(name),
                    Cell::new(description),
                    Cell::new(stars),
                    Cell::new(official),
                ]);
            }
            
            println!("{}", table);
        }
        
        ImageCommands::History { image } => {
            let history = docker.image_history(&image).await?;
            
            let mut table = Table::new();
            table.set_header(vec!["Created", "Created By", "Size"]);
            
            for item in history {
                let created = chrono::DateTime::from_timestamp(item.created, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default();
                let created_by = item.created_by.clone();
                let size = utils::format_bytes(item.size);
                
                table.add_row(vec![
                    Cell::new(created),
                    Cell::new(created_by),
                    Cell::new(size),
                ]);
            }
            
            println!("{}", table);
        }
        
        ImageCommands::Prune { all } => {
            let (space, images) = docker.prune_images(all).await?;
            println!("Removed {} images", images.len());
            println!("Reclaimed {}", utils::format_bytes(space as i64));
        }
    }
    
    Ok(())
}

async fn handle_volume_command(docker: DockerManager, command: VolumeCommands) -> Result<()> {
    match command {
        VolumeCommands::List => {
            let volumes = docker.list_volumes().await?;
            
            let mut table = Table::new();
            table.set_header(vec!["Name", "Driver", "Mountpoint"]);
            
            for volume in volumes {
                table.add_row(vec![
                    Cell::new(&volume.name),
                    Cell::new(&volume.driver),
                    Cell::new(&volume.mountpoint),
                ]);
            }
            
            println!("{}", table);
        }
        
        VolumeCommands::Create { name, driver } => {
            docker.create_volume(&name, Some(&driver), None).await?;
            println!("Volume '{}' created", name);
        }
        
        VolumeCommands::Remove { volume, force } => {
            docker.remove_volume(&volume, force).await?;
            println!("Volume '{}' removed", volume);
        }
        
        VolumeCommands::Inspect { volume } => {
            let vol = docker.inspect_volume(&volume).await?;
            println!("{}", serde_json::to_string_pretty(&vol)?);
        }
        
        VolumeCommands::Prune => {
            let (space, volumes) = docker.prune_volumes().await?;
            println!("Removed {} volumes", volumes.len());
            println!("Reclaimed {}", utils::format_bytes(space as i64));
        }
    }
    
    Ok(())
}

async fn handle_network_command(docker: DockerManager, command: NetworkCommands) -> Result<()> {
    match command {
        NetworkCommands::List => {
            let networks = docker.list_networks().await?;
            
            let mut table = Table::new();
            table.set_header(vec!["ID", "Name", "Driver", "Scope"]);
            
            for network in networks {
                let id = network.id.as_deref().unwrap_or("").chars().take(12).collect::<String>();
                let name = network.name.as_deref().unwrap_or("");
                let driver = network.driver.as_deref().unwrap_or("");
                let scope = network.scope.as_deref().unwrap_or("");
                
                table.add_row(vec![
                    Cell::new(id),
                    Cell::new(name),
                    Cell::new(driver),
                    Cell::new(scope),
                ]);
            }
            
            println!("{}", table);
        }
        
        NetworkCommands::Create { name, driver, internal, attachable } => {
            let id = docker.create_network(&name, Some(&driver), internal, attachable, None).await?;
            println!("Network '{}' created with ID: {}", name, id);
        }
        
        NetworkCommands::Remove { network } => {
            docker.remove_network(&network).await?;
            println!("Network '{}' removed", network);
        }
        
        NetworkCommands::Connect { container, network } => {
            docker.connect_container_to_network(&container, &network, None).await?;
            println!("Connected '{}' to network '{}'", container, network);
        }
        
        NetworkCommands::Disconnect { container, network, force } => {
            docker.disconnect_container_from_network(&container, &network, force).await?;
            println!("Disconnected '{}' from network '{}'", container, network);
        }
        
        NetworkCommands::Inspect { network } => {
            let net = docker.inspect_network(&network).await?;
            println!("{}", serde_json::to_string_pretty(&net)?);
        }
        
        NetworkCommands::Prune => {
            let networks = docker.prune_networks().await?;
            println!("Removed {} networks", networks.len());
        }
    }
    
    Ok(())
}

async fn handle_system_command(docker: DockerManager, command: SystemCommands) -> Result<()> {
    match command {
        SystemCommands::Version => {
            let version = docker.version().await?;
            println!("Docker version: {}", version.version.unwrap_or_default());
            println!("API version:    {}", version.api_version.unwrap_or_default());
            println!("OS/Arch:        {}/{}", 
                version.os.unwrap_or_default(), 
                version.arch.unwrap_or_default());
        }
        
        SystemCommands::Info => {
            let info = docker.info().await?;
            println!("Containers:     {} ({} running)", 
                info.containers.unwrap_or(0),
                info.containers_running.unwrap_or(0));
            println!("Images:         {}", info.images.unwrap_or(0));
            println!("Server Version: {}", info.server_version.unwrap_or_default());
            println!("Storage Driver: {}", info.driver.unwrap_or_default());
            println!("Docker Root:    {}", info.docker_root_dir.unwrap_or_default());
        }
        
        SystemCommands::Df => {
            let usage = docker.disk_usage().await?;
            
            println!("TYPE          TOTAL    ACTIVE    SIZE       RECLAIMABLE");
            
            if let Some(images) = usage.images {
                let total = images.len();
                let active = images.iter().filter(|i| i.containers > 0).count();
                let size: i64 = images.iter().map(|i| i.size).sum();
                let reclaimable: i64 = images.iter()
                    .filter(|i| i.containers == 0)
                    .map(|i| i.size)
                    .sum();
                
                println!("Images        {:<8} {:<9} {:<10} {} ({}%)",
                    total, active, 
                    utils::format_bytes(size),
                    utils::format_bytes(reclaimable),
                    if size > 0 { reclaimable * 100 / size } else { 0 });
            }
            
            if let Some(containers) = usage.containers {
                let total = containers.len();
                let running = containers.iter()
                    .filter(|c| c.state.as_deref() == Some("running"))
                    .count();
                let size: i64 = containers.iter()
                    .filter_map(|c| c.size_rw)
                    .sum();
                
                println!("Containers    {:<8} {:<9} {:<10}",
                    total, running, utils::format_bytes(size));
            }
            
            if let Some(volumes) = usage.volumes {
                let total = volumes.len();
                let active = volumes.iter()
                    .filter(|v| v.usage_data.as_ref()
                        .map(|u| u.ref_count > 0)
                        .unwrap_or(false))
                    .count();
                let size: i64 = volumes.iter()
                    .filter_map(|v| v.usage_data.as_ref()
                        .map(|u| u.size))
                    .sum();
                
                println!("Volumes       {:<8} {:<9} {:<10}",
                    total, active, utils::format_bytes(size));
            }
        }
        
        SystemCommands::Prune => {
            let result = docker.prune_all().await?;
            
            println!("Deleted Containers: {}", result.containers_deleted);
            println!("Deleted Images:     {}", result.images_deleted);
            println!("Deleted Volumes:    {}", result.volumes_deleted);
            println!("Deleted Networks:   {}", result.networks_deleted);
            println!();
            println!("Total reclaimed space: {}", 
                utils::format_bytes(result.total_space_reclaimed as i64));
        }
        
        SystemCommands::Health => {
            let health = docker.health_check().await?;
            
            if health.healthy {
                println!("Docker daemon is healthy");
            } else {
                println!("Docker daemon is NOT healthy");
            }
            
            println!();
            println!("Status:");
            println!("  Daemon responsive: {}", health.daemon_responsive);
            println!("  Total containers:  {}", health.total_containers);
            println!("  Running:           {}", health.running_containers);
            println!("  Total images:      {}", health.total_images);
            println!("  Disk usage:        {}", utils::format_bytes(health.disk_usage_bytes as i64));
            
            if !health.warnings.is_empty() {
                println!();
                println!("Warnings:");
                for warning in &health.warnings {
                    println!("  - {}", warning);
                }
            }
            
            if !health.errors.is_empty() {
                println!();
                println!("Errors:");
                for error in &health.errors {
                    println!("  - {}", error);
                }
            }
        }
    }
    
    Ok(())
}

async fn handle_compose_command(docker: DockerManager, command: ComposeCommands) -> Result<()> {
    use std::path::Path;
    
    match command {
        ComposeCommands::Up { dir, detach, services } => {
            let path = Path::new(&dir);
            let services: Option<Vec<&str>> = if services.is_empty() {
                None
            } else {
                Some(services.iter().map(|s| s.as_str()).collect())
            };
            
            docker.compose_up(path, detach, services).await?;
            println!("Services started");
        }
        
        ComposeCommands::Down { dir, volumes, remove_orphans } => {
            let path = Path::new(&dir);
            docker.compose_down(path, volumes, remove_orphans).await?;
            println!("Services stopped");
        }
        
        ComposeCommands::Logs { dir, follow, tail, services } => {
            let path = Path::new(&dir);
            let services: Option<Vec<&str>> = if services.is_empty() {
                None
            } else {
                Some(services.iter().map(|s| s.as_str()).collect())
            };
            
            if follow {
                docker.compose_logs_stream(path, services, |line| {
                    println!("{}", line);
                }).await?;
            } else {
                let logs = docker.compose_logs(path, false, tail.as_deref(), services).await?;
                print!("{}", logs);
            }
        }
        
        ComposeCommands::Ps { dir } => {
            let path = Path::new(&dir);
            let output = docker.compose_ps(path).await?;
            print!("{}", output);
        }
        
        ComposeCommands::Restart { dir, services } => {
            let path = Path::new(&dir);
            let services: Option<Vec<&str>> = if services.is_empty() {
                None
            } else {
                Some(services.iter().map(|s| s.as_str()).collect())
            };
            
            docker.compose_restart(path, services).await?;
            println!("Services restarted");
        }
        
        ComposeCommands::Build { dir, no_cache, services } => {
            let path = Path::new(&dir);
            let services: Option<Vec<&str>> = if services.is_empty() {
                None
            } else {
                Some(services.iter().map(|s| s.as_str()).collect())
            };
            
            docker.compose_build(path, no_cache, services).await?;
            println!("Services built");
        }
    }
    
    Ok(())
}

async fn handle_build_command(
    docker: DockerManager,
    context: String,
    tag: String,
    dockerfile: Option<String>,
    build_args: Vec<String>,
    no_cache: bool,
) -> Result<()> {
    use std::path::Path;
    
    let context_path = Path::new(&context);
    let build_args = utils::parse_labels(build_args);
    
    let progress = indicatif::ProgressBar::new_spinner();
    progress.set_message("Building image...");
    
    let image_id = docker.build_image_with_progress(
        context_path,
        dockerfile.as_deref(),
        &tag,
        Some(build_args),
        no_cache,
        |build_progress| {
            if let Some(step) = &build_progress.step {
                progress.set_message(step.trim().to_string());
                progress.tick();
            }
            if let Some(error) = &build_progress.error {
                progress.finish_with_message(format!("Error: {}", error));
            }
        },
    ).await?;
    
    progress.finish_with_message("Build complete");
    println!("Successfully built image '{}' with ID: {}", tag, image_id);
    
    Ok(())
}