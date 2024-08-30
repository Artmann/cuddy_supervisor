# Cuddy Supervisor

Welcome to the `cuddy_supervisor` repository! This is the supervisor server for **Cuddy**, a robust and efficient background job system. The `cuddy_supervisor` is responsible for keeping track of jobs and delegating them to workers, ensuring that your background tasks are managed effectively and reliably.

## Features

- **Job Tracking**: Keeps a record of all jobs, their status, and their results.
- **Worker Delegation**: Efficiently delegates jobs to available workers, optimizing resource usage.
- **Fault Tolerance**: Handles job failures and retries gracefully.

## Getting Started

N/A

## Development

### Prerequisites

Before you begin, ensure you have met the following requirements:

- Rust toolchain installed (version 1.XX or higher).
- SQLite installed.

### Installation

To get started with `cuddy_supervisor`, clone this repository and build the project using Cargo:

```bash
git clone https://github.com/artmann/cuddy_supervisor.git
cd cuddy_supervisor

cargo build --release

```

### Configuration

Copy `.env.example` to `.env`.

### Running the Server

Once configured, you can start the supervisor server:

```bash
cargo run
```

This will start the supervisor and it will begin monitoring the job queue, delegating tasks to workers as they become available.

## API

N/A

## Contributing

Contributions are welcome! If you have suggestions or want to contribute new features, feel free to open a pull request or file an issue.

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/new-feature`).
3. Commit your changes (`git commit -m 'Add new feature'`).
4. Push to the branch (`git push origin feature/new-feature`).
5. Open a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE.md) file for more details.

Thank you for using `cuddy_supervisor`! We hope it serves you well in managing your background jobs efficiently and reliably.
