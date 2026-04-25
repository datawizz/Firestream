#!/bin/bash

# Multi-Platform App Docker Cleanup Script
# This script removes Docker containers, images, networks, and volumes associated with the project
# It also handles VS Code devcontainer cleanup

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
PROJECT_NAME="multi_platform_app"
COMPOSE_PROJECT="multi_platform_appco"
DRY_RUN=false
FORCE=false
CLEAN_ALL=true
CLEAN_CONTAINERS=true
CLEAN_IMAGES=true
CLEAN_NETWORKS=true
CLEAN_VOLUMES=true

# Usage function
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Docker cleanup script for multi_platform_app project

OPTIONS:
    -h, --help              Show this help message
    -f, --force             Skip confirmation prompts
    -d, --dry-run           Show what would be cleaned without doing it
    -c, --containers-only   Only clean containers
    -i, --images-only       Only clean images
    -n, --networks-only     Only clean networks
    -v, --volumes-only      Only clean volumes
    --all                   Clean all resources (default)

EXAMPLES:
    $0                      # Interactive cleanup of all resources
    $0 -f                   # Force cleanup without prompts
    $0 -d                   # Dry run to see what would be cleaned
    $0 -c                   # Clean only containers

EOF
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -c|--containers-only)
            CLEAN_ALL=false
            CLEAN_CONTAINERS=true
            CLEAN_IMAGES=false
            CLEAN_NETWORKS=false
            CLEAN_VOLUMES=false
            shift
            ;;
        -i|--images-only)
            CLEAN_ALL=false
            CLEAN_CONTAINERS=false
            CLEAN_IMAGES=true
            CLEAN_NETWORKS=false
            CLEAN_VOLUMES=false
            shift
            ;;
        -n|--networks-only)
            CLEAN_ALL=false
            CLEAN_CONTAINERS=false
            CLEAN_IMAGES=false
            CLEAN_NETWORKS=true
            CLEAN_VOLUMES=false
            shift
            ;;
        -v|--volumes-only)
            CLEAN_ALL=false
            CLEAN_CONTAINERS=false
            CLEAN_IMAGES=false
            CLEAN_NETWORKS=false
            CLEAN_VOLUMES=true
            shift
            ;;
        --all)
            CLEAN_ALL=true
            CLEAN_CONTAINERS=true
            CLEAN_IMAGES=true
            CLEAN_NETWORKS=true
            CLEAN_VOLUMES=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

# Function to print colored messages
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to execute or simulate commands
execute_command() {
    local cmd=$1
    local description=$2

    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN]${NC} Would execute: $cmd"
        echo "          Purpose: $description"
    else
        print_info "$description"
        eval "$cmd" || true
    fi
}

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed or not in PATH"
    exit 1
fi

# Print header
echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}  Multi-Platform App Docker Cleanup Tool ${NC}"
echo -e "${YELLOW}========================================${NC}"
echo

# Show what will be cleaned
print_info "Cleanup targets:"
[ "$CLEAN_CONTAINERS" = true ] && echo "  ✓ Containers"
[ "$CLEAN_IMAGES" = true ] && echo "  ✓ Images"
[ "$CLEAN_NETWORKS" = true ] && echo "  ✓ Networks"
[ "$CLEAN_VOLUMES" = true ] && echo "  ✓ Volumes"
echo

# Dry run notice
if [ "$DRY_RUN" = true ]; then
    print_warning "DRY RUN MODE - No changes will be made"
    echo
fi

# Confirmation prompt
if [ "$FORCE" = false ] && [ "$DRY_RUN" = false ]; then
    print_warning "This will remove Docker resources for the multi_platform_app project"
    echo -e "${YELLOW}This includes:${NC}"
    echo "  - All containers with 'multi_platform_app' or 'multi_platform_appco' in their name"
    echo "  - All VS Code devcontainer containers for this project"
    echo "  - Docker images: multi_platform_app:*, project:latest (legacy)"
    echo "  - Networks and volumes associated with the project"
    echo
    read -p "Do you want to continue? (Y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Operation cancelled by user"
        exit 0
    fi
fi

echo

# Clean containers
if [ "$CLEAN_CONTAINERS" = true ]; then
    print_info "Cleaning containers..."

    # Stop and remove containers by name pattern
    containers=$(docker ps -aq --filter "name=${COMPOSE_PROJECT}" --filter "name=${PROJECT_NAME}" 2>/dev/null || true)
    if [ -n "$containers" ]; then
        execute_command "docker stop $containers 2>/dev/null" "Stopping containers with project name"
        execute_command "docker rm $containers 2>/dev/null" "Removing containers with project name"
    fi

    # Stop and remove VS Code devcontainers
    vscode_containers=$(docker ps -aq --filter "label=com.docker.compose.project=${COMPOSE_PROJECT}" --filter "label=com.docker.compose.service=devcontainer" 2>/dev/null || true)
    if [ -n "$vscode_containers" ]; then
        execute_command "docker stop $vscode_containers 2>/dev/null" "Stopping VS Code devcontainers"
        execute_command "docker rm $vscode_containers 2>/dev/null" "Removing VS Code devcontainers"
    fi

    # Also check for any containers with the old "project" name
    old_containers=$(docker ps -aq --filter "name=project" 2>/dev/null || true)
    if [ -n "$old_containers" ]; then
        execute_command "docker stop $old_containers 2>/dev/null" "Stopping legacy 'project' containers"
        execute_command "docker rm $old_containers 2>/dev/null" "Removing legacy 'project' containers"
    fi

    print_success "Container cleanup complete"
    echo
fi

# Clean images
if [ "$CLEAN_IMAGES" = true ]; then
    print_info "Cleaning images..."

    # Remove multi_platform_app images
    multi_platform_app_images=$(docker images -q "${PROJECT_NAME}:*" 2>/dev/null || true)
    if [ -n "$multi_platform_app_images" ]; then
        execute_command "docker rmi -f $multi_platform_app_images 2>/dev/null" "Removing multi_platform_app:* images"
    fi

    # Remove legacy project:latest image
    project_image=$(docker images -q "project:latest" 2>/dev/null || true)
    if [ -n "$project_image" ]; then
        execute_command "docker rmi -f $project_image 2>/dev/null" "Removing legacy project:latest image"
    fi

    # Remove any images with multi_platform_app in the name
    other_images=$(docker images | grep -E "(${PROJECT_NAME}|${COMPOSE_PROJECT})" | awk '{print $3}' 2>/dev/null || true)
    if [ -n "$other_images" ]; then
        execute_command "docker rmi -f $other_images 2>/dev/null" "Removing other project-related images"
    fi

    print_success "Image cleanup complete"
    echo
fi

# Clean networks
if [ "$CLEAN_NETWORKS" = true ]; then
    print_info "Cleaning networks..."

    # Remove networks
    networks=$(docker network ls -q --filter "name=${COMPOSE_PROJECT}" 2>/dev/null || true)
    if [ -n "$networks" ]; then
        execute_command "docker network rm $networks 2>/dev/null" "Removing project networks"
    fi

    print_success "Network cleanup complete"
    echo
fi

# Clean volumes
if [ "$CLEAN_VOLUMES" = true ]; then
    print_info "Cleaning volumes..."

    # Remove volumes
    volumes=$(docker volume ls -q --filter "name=${COMPOSE_PROJECT}" 2>/dev/null || true)
    if [ -n "$volumes" ]; then
        execute_command "docker volume rm $volumes 2>/dev/null" "Removing project volumes"
    fi

    # Remove VS Code volumes
    vscode_volumes=$(docker volume ls -q --filter "name=vscode" 2>/dev/null || true)
    if [ -n "$vscode_volumes" ]; then
        print_warning "Found VS Code volumes. These may be used by other projects."
        if [ "$FORCE" = true ] || [ "$DRY_RUN" = true ]; then
            execute_command "docker volume rm $vscode_volumes 2>/dev/null" "Removing VS Code volumes"
        else
            read -p "Remove VS Code volumes? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                execute_command "docker volume rm $vscode_volumes 2>/dev/null" "Removing VS Code volumes"
            fi
        fi
    fi

    print_success "Volume cleanup complete"
    echo
fi

# Final cleanup - prune dangling resources
if [ "$DRY_RUN" = false ] && [ "$FORCE" = true ]; then
    print_info "Pruning dangling resources..."
    docker system prune -f > /dev/null 2>&1
    print_success "System prune complete"
elif [ "$DRY_RUN" = false ]; then
    print_info "Run 'docker system prune' to clean up any remaining dangling resources"
fi

# Summary
echo
echo -e "${GREEN}========================================${NC}"
if [ "$DRY_RUN" = true ]; then
    echo -e "${GREEN}  Dry run complete - no changes made    ${NC}"
else
    echo -e "${GREEN}  Docker cleanup complete!              ${NC}"
fi
echo -e "${GREEN}========================================${NC}"

# Provide next steps
echo
print_info "Next steps:"
echo "  1. Rebuild the development container: make dev"
echo "  2. Or use VS Code: Reopen in Container"
echo "  3. Check Docker status: docker ps -a"
echo

exit 0
