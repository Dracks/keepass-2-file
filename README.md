# keepass-2-file

A command-line tool to generate environment files using KeePass databases as a secure source for sensitive values.

## Features

- Generate `.env` files, or any other text/plain file, from templates using Handlebars syntax
- Securely retrieve credentials from KeePass databases
- Support for multiple KeePass entry fields (password, username, URL, and additional attributes)
- Global configuration for default KeePass database location
- Flexible output path handling (absolute or relative paths)

## Installation

### From brew
```bash
brew install dracks/homebrew-dracks/keepass-2-file
```

### From the source code
```bash
git clone git@github.com:Dracks/keepass-2-file
cd keepass-2-file
cargo install --path .
```

### Downloading the last executable

1. Download the executable from the repo
2. Place it into a folder that you have in your PATH

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

# delete the template with name template-name on the non-global config specified
keepass-2-file --config non/global/config/file.yaml config delete name template-name
```

### Build Command Options

```bash
keepass-2-file build <TEMPLATE> <OUTPUT> [OPTIONS]

Options:
  -k, --keepass <FILE>      Overwrite the global keepass file
  -r, --relative-to-input   Make output path relative to template location
  -p, --password <Password> Uses the password as keepass password
```

### Config Command Options

```bash
keepass-2-file config <COMMAND> [ARGS]

Commands:
    set-default-kp-db  <PATH>        Set the default KeePass file
    get-kp-db                        Get the current KeePass file
    list-files                       List the templates inside the configuration
    add-file <SOURCE> <DESTINATION>  Add a template into the config
    prune                            Deletes all templates that the source doesn't exists
    delete                           Deletes a template
    help                             Print this message or the help of the given subcommand(s)
```

### Using Preconfigured files

```bash
# Add a commonly used template to your configuration
keepass-2-file config add-file folder/templates/dev-template.hbs .env --name dev-template

# List all configured files
keepass-2-file config list-files

# Will build all the templates configured in the config
keepass-2-file build-all
```

**note**: the adding of files support the same flat -r/--relative-to-input to not specify the full path two times

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
