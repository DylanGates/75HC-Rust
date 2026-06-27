# Rust Todo CLI - Commands Reference

A powerful command-line todo application built with Rust, featuring JSON storage, due dates, search, and statistics.

## Installation & Setup

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Building the Project

```bash
# Clone or navigate to the project directory
cd /Users/admin/Projects/75HC/75HC-Rust/todo_cli

# Build the project
cargo build

# Or build optimized release version
cargo build --release
```

### Running the Application

```bash
# Run in development mode
cargo run -- <command> <arguments>

# Run release version
cargo run --release -- <command> <arguments>

# Or run the compiled binary directly
./target/debug/todo_cli <command> <arguments>
./target/release/todo_cli <command> <arguments>
```

## Available Commands

### 1. Add a New Task

```bash
cargo run -- add "task description" --priority <low|medium|high>
```

**Examples:**

```bash
cargo run -- add "finish project report"
cargo run -- add "buy groceries" --priority high
cargo run -- add "call dentist" --priority low
```

### 2. List All Tasks

```bash
cargo run -- list
```

Shows all tasks with their status, priority, and creation date.

### 3. List Scheduled Tasks

```bash
cargo run -- scheduled
```

Shows only tasks that have due dates set.

### 4. Mark Task as Complete

```bash
cargo run -- complete "task description"
```

**Example:**

```bash
cargo run -- complete "finish project report"
```

### 5. Set Due Date for a Task

```bash
cargo run -- due "task description" "YYYY-MM-DD" [HH:MM]
```

**Examples:**

```bash
cargo run -- due "finish project report" "2025-12-25"
cargo run -- due "call dentist" "2025-12-22 14:30"
```

### 6. Search Tasks

```bash
cargo run -- search "keyword"
```

**Example:**

```bash
cargo run -- search "project"
```

### 7. Delete a Task

```bash
cargo run -- delete "task description"
```

**Example:**

```bash
cargo run -- delete "buy groceries"
```

### 8. Show Statistics

```bash
cargo run -- stats
```

Displays comprehensive statistics including:

- Total tasks count
- Completed vs pending tasks
- Completion percentage
- Number of scheduled tasks
- Priority breakdown

### 9. Help

```bash
cargo run -- --help
cargo run -- help <command>
```

## Command Reference Table

| Command     | Description               | Example                                   |
| ----------- | ------------------------- | ----------------------------------------- |
| `add`       | Add new task              | `cargo run -- add "task" --priority high` |
| `list`      | List all tasks            | `cargo run -- list`                       |
| `scheduled` | List tasks with due dates | `cargo run -- scheduled`                  |
| `complete`  | Mark task complete        | `cargo run -- complete "task"`            |
| `due`       | Set due date              | `cargo run -- due "task" "2025-12-25"`    |
| `search`    | Search tasks by keyword   | `cargo run -- search "keyword"`           |
| `delete`    | Delete a task             | `cargo run -- delete "task"`              |
| `stats`     | Show statistics           | `cargo run -- stats`                      |

## Features

- ✅ **JSON Storage**: Tasks are stored in `todo.json` file
- ✅ **Priority Levels**: Low, Medium, High priorities
- ✅ **Due Dates**: Set and track due dates for tasks
- ✅ **Search Functionality**: Find tasks by keywords
- ✅ **Task Completion**: Mark tasks as done
- ✅ **Statistics**: View completion rates and breakdowns
- ✅ **Scheduled Tasks**: Filter and view tasks with due dates
- ✅ **Timestamps**: Automatic creation timestamps
- ✅ **Persistent Storage**: Data survives application restarts

## Data Storage

Tasks are stored in `todo.json` in the project root directory. Each task contains:

- Unique ID
- Description
- Priority (Low/Medium/High)
- Completion status
- Creation timestamp
- Due date (optional)

## Common Workflows

### Create a Scheduled Task

```bash
# Add task
cargo run -- add "finish quarterly report" --priority high

# Set due date
cargo run -- due "finish quarterly report" "2025-12-31"

# View scheduled tasks
cargo run -- scheduled
```

### Daily Task Management

```bash
# Add multiple tasks
cargo run -- add "morning standup" --priority medium
cargo run -- add "code review" --priority high
cargo run -- add "update documentation" --priority low

# View all tasks
cargo run -- list

# Mark completed tasks
cargo run -- complete "morning standup"

# Check progress
cargo run -- stats
```

### Search and Filter

```bash
# Search for specific tasks
cargo run -- search "code"

# View only scheduled tasks
cargo run -- scheduled
```

## Error Handling

The application provides helpful error messages for:

- Invalid date formats (use YYYY-MM-DD or YYYY-MM-DD HH:MM)
- Task not found errors
- File I/O errors
- Invalid priority levels

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Dependencies

- `clap`: Command-line argument parsing
- `serde`: JSON serialization/deserialization
- `chrono`: Date and time handling

## Version History

- **v0.1.0**: Initial release with basic CRUD operations
- Added JSON storage, due dates, search, delete, and statistics features
- Improved CLI interface with clap

---

For more information, run `cargo run -- --help` or `cargo run -- help <command>`
