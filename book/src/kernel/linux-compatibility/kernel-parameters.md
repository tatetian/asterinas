# Kernel Parameters

This section documents kernel command-line parameters supported by Asterinas.

## Inherited from Linux

### `init`

Run the specified binary as `init`.

Example:
```text
init=/bin/busybox
```

Notes:
- The value is the path to the executable.
- If omitted, Asterinas will try to execute from the following paths in order:
  `/sbin/init`, `/etc/init`, `/bin/init`, `/bin/sh`.

### `console`

Select console devices for kernel messages.
This parameter may be specified multiple times.
Kernel messages are delivered to each listed console.

Valid values:
- `tty0`
- `ttyS0`
- `hvc0`

Examples:
```text
console=ttyS0
console=ttyS0 console=hvc0
```

### `earlycon`

Enable the early console that OSTD brings up during early boot.
The name follows Linux's `earlycon` parameter.
Asterinas currently supports a simplified form.

Example:
```text
earlycon
```

Notes:
- If omitted, the early console stays disabled.
- Only the bare `earlycon` token is supported; complex Linux forms such as `earlycon=uart8250,io,0x3f8,115200` are not supported yet.

### `loglevel`

Set the OSTD log filter (`ostd::log::LevelFilter`).

Valid values are decimal integers `0` through `8`:

| Value | `LevelFilter` | Messages printed                    |
|-------|---------------|-------------------------------------|
| `0`   | `Off`         | (none)                              |
| `1`   | `Emerg`       | emerg only                          |
| `2`   | `Alert`       | emerg, alert                        |
| `3`   | `Crit`        | emerg through crit                  |
| `4`   | `Error`       | emerg through error                 |
| `5`   | `Warning`     | emerg through warn                  |
| `6`   | `Notice`      | emerg through notice                |
| `7`   | `Info`        | emerg through info                  |
| `8`   | `Debug`       | all levels (emerg through debug)    |

Example:
```text
loglevel=4
```

Notes:
- When omitted, invalid, or greater than `8`, the default is `8` (debug).

Combined example:
```text
earlycon loglevel=4 console=ttyS0
```

## Asterinas-specific

### `i8042.exist`

Override ACPI's indication of whether a PS/2 (i8042) controller exists.

Valid values:
- `1`, `on`, `yes`, `true` or no value — treat the i8042 controller as present (force probing)
- `0`, `off`, `no`, `false` - treat the i8042 controller as absent (skip probing)

Examples:
```text
i8042.exist
i8042.exist=1
i8042.exist=0
```
