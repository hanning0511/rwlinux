# Read Write on Linux

## Build

1. Install Rust toolchain

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Build and install the executables

   ```sh
   make
   make install
   ```

## `devmem`

![](images/devmem.PNG)

Root privileges are required to run `devmem`.

### Navigation

You can browse the data in `/dev/mem` by changing the offset value.
The following table lists the key bindings that change the offset value.

|Operation|Key(s)|
|-|-|
|Next Cell|`l` or `ArrowRight`|
|Previous Cell|`h` or `ArrowLeft`|
|Next Line|`k` or `ArrowDown`|
|Previous Line|`j` or `ArrowUp`|
|Next Page|`n` or `PageDown`|
|Previous Page|`p` or `PageUp`|

In addition to above key bindings, you can also jump to specified offset.
Press `J`, enter the offset, then press `Enter`.
2 types of offset are supported:

- Absolute offset. For example: `E0000000`.
- Relative offset, relative to current offset. For example: `+FFF`, `-FF`.

### Data Type

By defaut, data is displayed in bytes. The data type can be switched with following key bindings.

|Data Type|Key|
|-|-|
|Byte|`B`|
|Word|`W`|
|Double Word|`D`|
|Quad Word|`Q`|

### Data Write

In addition to browsingdata in `/dev/mem`, **devmem** also supports for writing data to `/dev/mem`. You can do this following below steps:

1. Change to an offset.
2. Press `e`.
3. Input the data to be written to the offset.
   - Specify data type by using below prefixes:
      |Prefix|Data Type|Example|
      |-|-|-|
      |**B:**|Byte|B:FF|
      |**W:**|Word|W:EF78|
      |**DW:**|Double Word|DW:12345678|
      |**QW:**|Quad Word|QW:1234567887654321|
      |**DQW:**|Double Quad Word|DQW:12345678876543211234567887654321|
   - Data will be written to `/dev/mem` byte by byte if data type is not specified.
4. Press `Enter`.
