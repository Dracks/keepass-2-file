<p align="center">
  <img src="kp2f.png" width="100" height="100"/>
</p>

<p align="center">
  <a href="https://github.com/Dracks/keepass-2-file/actions/workflows/rust.yml">
    <img src="https://github.com/Dracks/keepass-2-file/actions/workflows/rust.yml/badge.svg?branch=main" />
  </a>
  <a href="https://codecov.io/gh/Dracks/keepass-2-file">
    <img src="https://codecov.io/gh/Dracks/keepass-2-file/branch/main/graph/badge.svg" />
  </a>
</p>

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
# will download and build it from code installing rust if necessary
brew install dracks/homebrew-dracks/keepass-2-file

# Will download the binary from the release
brew install dracks/homebrew-dracks/keepass-2-file-bin
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
keepass-2-file config set-default-kp-db /path/to/your.kdbx

# Get current KeePass database configuration
keepass-2-file config get-kp-db

# Generate environment file from template
keepass-2-file build template.env.hbs .env

# Using relative output path, will generate the file in /some-project/envs/aws.env
keepass-2-file build /some-project/devops/aws.env.hbs --relative-to-input ../envs/aws.env

# Delete the template with name template-name on the non-global config specified
keepass-2-file --config non/global/config/file.yaml config delete name template-name

# Build some .env overwriting or adding multiple variables
keepass-2-file build -v "email=j@k.com" -v "other=daleks in manhattam" file.env.example .env
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
    add-variables <var1=value>...    Add one or more default global variables
    delete-variables <var1>...        Delete one or more default global variables
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

### Using the global variables

Having global variables can be useful in multiple cases, but one good example is to have an e-mail, in the .env for having your own e-mail when testing sending e-mails, and not spam your team e-mail list. Another use can be to have some configurations, that you wish to be able to disable for your machine, but other collegues needs to have it enable/or customize it

```bash
# Adding multiple variables
keepass-2-file config add-variables var=first 'var2=Some sample of variable'

#deleting multiple variables
keepass-2-file config delete-variables var1 var2
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

# Using a local variable
EMAIL={{stringify email}}
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
