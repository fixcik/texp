
`texp` is a compact tool designed for rendering templates in a syntax reminiscent of Jinja, powered by [Tera](https://github.com/Keats/). It's useful for generating text output based on predefined templates and variable files.

Learn more about template syntax on the [Tera Documentation](https://keats.github.io/tera/docs/).

Table of Contents:

- [Examples](#examples)
- [Installation](#installation)
  - [Via cargo](#via-cargo)

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

### Via cargo

```bash
cargo install texp
```
