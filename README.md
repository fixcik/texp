`texp` is a compact tool designed for rendering templates in a syntax reminiscent of Jinja, powered by [Tera](https://github.com/Keats/). It's useful for generating text output based on predefined templates and variable files.

Learn more about template syntax on the [Tera Documentation](https://keats.github.io/tera/docs/).

```
sage: texp [OPTIONS] <path>

Arguments:
  <path>

Options:
  -v, --values <VALUES>  Path to yaml file with values
      --set <SET>        Set value, e.g "--set foo.a=bar --set foo.b=baz"
  -o, --output <OUTPUT>  Path to output file
  -h, --help             Print help
  -V, --version          Print version
```

Table of Contents:

- [Examples](#examples)
- [Installation](#installation)
  - [Prebuilt Binaries](#prebuilt-binaries)
  - [Via cargo](#via-cargo)
  - [Via homebrew](#via-homebrew)
- [Custom functions](#custom-functions)
  - [randomString](#randomstring)
  - [jwtToken](#jwttoken)

## Examples

Below is a simple demonstration of how `texp` works. Start by creating two files: one for variables and one for the template.

variables.yaml:

```yaml
foo: bar
```

doc.template.html:

```html
<p>{{ foo }}</p>
```

Running texp with these files as inputs:

```bash
texp -v variables.yaml doc.template.html
```

produces the output:

```html
<p>bar</p>
```

Also you can set output flag, to save result to file:

```bash
texp -v variables.yaml -o doc.html doc.template.html
```

## Installation

### Prebuilt Binaries

Download the latest releases from the [GitHub release page](https://github.com/fixcik/texp/releases).

### Via cargo

```bash
cargo install texp
```

### Via homebrew

```bash
brew tap fixcik/tap
brew install texp
```

## Custom functions

### randomString

Generates random string, with given length:

```
{{ randomString(length=20) }}
```

### jwtToken

Generate jwt token

```
{{ jwtToken(claims=claims, secret=secret) }}
```
