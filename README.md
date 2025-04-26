# Git HTTP Server
## Description

This is a lightweight Git HTTP server implementation that allows you to host Git repositories with basic authentication and time-based write access control. It's built using Rust and Actix-web, providing a simple yet functional alternative to traditional Git hosting solutions.

The server implements the Git Smart HTTP protocol, allowing standard Git operations like clone, pull, and push while maintaining security through basic authentication. A unique feature is the time-based write access control, which restricts repository modifications to specific hours while maintaining read access 24/7.

### Key Features

- **Smart HTTP Protocol Support**: Full implementation of Git's Smart HTTP protocol
- **Time-Based Access Control**: Write operations restricted to 10:00-12:00 UTC
- **Basic Authentication**: Simple username/password authentication
- **Repository Browsing**: Direct access to repository files through HTTP
- **Pack File Support**: Efficient handling of Git pack files
- **Lightweight**: Minimal dependencies and simple configuration
- **Real-Time Access Control**: Write permissions based on current UTC time (what am i doing wrong in my life..)

## Features

- Basic Git operations support (clone, pull, push)
- Basic Authentication
- Time-based write access (10:00 - 12:00 UTC)
- Read-only access outside write window
- Pack files support
- Text file viewing support

## Configuration

### Default credentials
```
Username: Kazilsky
Password: password123
```

## Usage

### Start the server
```bash
cargo run
```

The server will start on `http://localhost:8000`

### Repository Operations

Clone a repository:
```bash
git clone http://localhost:8000/git/repo-name
# Or with credentials:
git clone http://Kazilsky:password123@localhost:8000/git/repo-name
```

Push changes (only during write window):
```bash
git push origin main
```

View repository files:
```bash
# View README.md
curl http://localhost:8000/git/repo-name/file/README.md

# View any text file
curl http://localhost:8000/git/repo-name/file/path/to/file.txt
```

## API Endpoints

- `GET /git/{repo_name}/info/refs` - Git protocol discovery
- `POST /git/{repo_name}/git-upload-pack` - Download objects (clone/pull)
- `POST /git/{repo_name}/git-receive-pack` - Upload objects (push)
- `GET /git/{repo_name}/objects/info/packs` - List available pack files
- `GET /git/{repo_name}/objects/pack/{pack_file}` - Download pack file
- `GET /git/{repo_name}/file/{path}` - View repository files

## Security

- Basic authentication required for all operations
- Write access restricted to configured time window
- Read access available 24/7
- Username/password stored in memory (for demonstration)

## Project Structure

```
src/
  └── main.rs         # Main server implementation
repositories/
  └── {repo}.git      # Git bare repositories
```

## Error Handling

- 401 Unauthorized - Authentication required
- 403 Forbidden - Write access denied (outside time window)
- 404 Not Found - Repository or file not found
- 500 Internal Server Error - Git operation failed

## Dependencies

- actix-web
- base64
- chrono
- lazy_static
- log
- env_logger

## Development

1. Clone the repository
2. Create a `repositories` directory
3. Initialize bare Git repositories in `repositories/{repo}.git`
4. Run with `cargo run`

## Notes

This is a demonstration project showing how to implement a basic Git HTTP server. For production use, consider:

- Using a proper database for user management
- Implementing more robust authentication
- Adding HTTPS support
- Adding proper logging
- Implementing repository access control
- Adding monitoring and metrics
- Adding rate limiting
- Adding backup solutions

## License

MIT License. See LICENSE file for details.
