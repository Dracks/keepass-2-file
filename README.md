# keepass-2-file

A command-line tool to generate environment files using KeePass databases as a secure source for sensitive values.

## Features

- Generate `.env` files, or any other text/plain file, from templates using Handlebars syntax
- Securely retrieve credentials from KeePass databases
- Support for multiple KeePass entry fields (password, username, URL, and additional attributes)
- Global configuration for default KeePass database location
- Flexible output path handling (absolute or relative paths)

## Installation

```bash
cargo install keepass-2-file
```

## Usage

### Basic Commands

```bash
# Set default KeePass database
keepass-2-file config set-default-kpdb /path/to/your.kdbx

# Get current KeePass database configuration
keepass-2-file config get-kpdb

# Generate environment file from template
keepass-2-file build template.env.hbs .env

# Using relative output path, will generate the file in /some-project/envs/aws.env
keepass-2-file build /some-project/devops/aws.env.hbs --relative-to-input ../envs/aws.env
```

### Build Command Options

```bash
keepass-2-file build <TEMPLATE> <OUTPUT> [OPTIONS]

Options:
  -k, --keepass <FILE>     Overwrite the global keepass file
  -r, --relative-to-input  Make output path relative to template location
```

## Template Syntax

The template uses Handlebars syntax with custom helpers to access KeePass entries:

### Basic Password Retrieval
```handlebars
DB_PASSWORD={{keepass "Entry"}}
```

### Accessing Specific Fields
```handlebars
DB_USERNAME={{keepass field=username "Entry"}}
DB_URL={{keepass field=url "Entry"}}
```

### Additional Attributes
```handlebars
CUSTOM_FIELD={{keepass field=field-name "Entry"}}
```

### String Escaping
```handlebars
ESCAPED_VALUE={{stringify (keepass "Entry")}}
```

## Example Template

```env
# Database Configuration
DB_HOST=localhost
DB_PORT=5432
DB_USER={{keepass field=username "PostgreSQL"}}
DB_PASSWORD={{keepass "PostgreSQL"}}

# API Keys
API_KEY={{stringify (keepass "APIs" "ExternalService")}}
```

## Configuration

The global configuration file is stored at `~/.config/keepass-2-file.yaml` by default. You can specify a different location using the global `--config` option.

## Error Messages

- `<Not found keepass entry>`: The specified KeePass entry path wasn't found
- `<No password found in entry>`: The entry exists but has no password
- `<No username found in entry>`: The entry exists but has no username
- `<No URL found in entry>`: The entry exists but has no URL
- `<Attribute not found in entry>`: The specified additional attribute wasn't found
- `<Invalid field type>`: An invalid field type was specified

## Development

### Running Tests

```bash
cargo test
```

### Building from Source

```bash
cargo build --release
```
