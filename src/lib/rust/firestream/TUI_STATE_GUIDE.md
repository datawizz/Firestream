# Firestream TUI - State Management Guide

## Overview

The Firestream TUI now includes integrated state management with plan/apply workflow similar to Terraform.

## Navigating the TUI

### Tab Navigation
- **Tab**: Move to next tab
- **Shift+Tab**: Move to previous tab
- **↑/k**: Move up in lists
- **↓/j**: Move down in lists
- **q**: Quit

### Available Tabs

#### 1. Services Tab
Original service management functionality:
- **i**: Install service
- **s**: Start/Stop service
- **r**: Restart service

#### 2. State Tab
View and manage Firestream state:
- **r**: Refresh state from disk
- **l**: Lock state (prevents concurrent modifications)
- **u**: Unlock state

Shows:
- State version and serial number
- Last modification info
- Infrastructure configuration
- Resource counts (builds, deployments, cluster)

#### 3. Plan Tab
Create and review execution plans:
- **p**: Create new plan (compares current state with desired configuration)
- **d**: Discard current plan
- **↑/↓**: Navigate through planned changes

Shows:
- List of changes (Create/Update/Delete/Replace)
- Resource IDs and descriptions
- Impact assessment (Low/Medium/High/Critical)

#### 4. Deploy Tab
Apply execution plans:
- **a**: Apply the plan
- **y**: Approve plan (if pending)
- **n**: Reject plan

Shows:
- Plan details (ID, creator, timestamp)
- Plan status (Pending/Approved/Applying/Applied/Failed)
- Deployment progress

## Workflow Example

1. **Start the TUI**:
   ```bash
   firestream tui
   ```

2. **Navigate to State tab** (press Tab)
   - Press 'r' to load current state
   - Review existing resources

3. **Navigate to Plan tab** (press Tab)
   - Press 'p' to create execution plan
   - Review proposed changes
   - Use ↑/↓ to examine specific changes

4. **Navigate to Deploy tab** (press Tab)
   - If plan requires approval, press 'y' to approve
   - Press 'a' to apply the plan
   - Monitor progress in the status area

## State File Location

State files are stored in:
- `.firestream/state/firestream.state.json` - Current state
- `.firestream/state/plans/` - Execution plans
- `.firestream/state/backups/` - State backups
- `.firestream/state/locks/` - Lock files

## Configuration

The TUI reads configuration from `firestream.toml` in the current directory. This file defines:
- Cluster settings
- Infrastructure requirements
- Build definitions
- Deployment specifications

## Tips

1. **Always refresh state** before creating plans to ensure you're working with the latest information
2. **Lock state** when performing critical operations to prevent conflicts
3. **Review plans carefully** before applying - look for high-impact changes
4. **Check Deploy tab status** for operation progress and errors

## Troubleshooting

- **"Failed to lock state"**: Another process has locked the state. Wait or use unlock if the lock is stale
- **"No state loaded"**: Navigate to State tab and press 'r' to refresh
- **"Plan must be approved"**: Navigate to Deploy tab and press 'y' to approve before applying
