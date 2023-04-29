# picbin

Commandline tool for bidirectional conversion between a binary file and an image.

**DISCLAIMER: This tool is not designed for serious use
and should only be used for experimentation or educational purposes.
Use at your own risk, and don't forget to make a backup!**

---

## Cheatsheet

Use `encode` command to encode a binary file into an image file:

```shell
picbin encode /source/path/to/original-binary-file.bin /dest/path/to/encoded.png
```

Run `decode` subcommand to extract the original content from the encoded image file:

```shell
picbin decode /path/to/the/encoded/image.png /dest/path/to/extract/original-binary-file.bin
```

In both cases you can use `--overwrite` switch if you want to do so:

```shell
picbin --overwrite decode /path/to/the/encoded/image.png /the/existent/file/will/be/overwritten.bin
```

You can see those usage with `help` command:

```shell
picbin help
picbin help encode
picbin help decode
```

## Specification

- A single byte is represented as a single colored pixel.
- Pixels are arranged in Z-pattern layout (from left to right, and then top to bottom).

### Color mapping
We define a mapping between byte and color by dividing HSV-based color wheel into 256 sections.
Each section corresponds to a byte pattern, from 0x00 to 0xFF.

Here's the concept (please note that values may not be accurate):

```
HUE(deg)    HEX     COLOR      SECTION    BYTE
--------  -------  -------    ---------  ------
    0     #FF0000   red        No.  0     0x00
    |        |       |            :        :
   60     #FFFF00   yellow        :        :
    |        |       |            :        :
  120     #00FF00   lime          :        :
    |        |       |            :        :
  180     #00FFFF   cyan       No.127     0x80
    |        |       |            :        :
  240     #0000FF   blue          :        :
    |        |       |            :        :
  300     #FF00FF   fuchsia       :        :
    |        |       |            :        :
  358     #FF00bF    |         No.255     0xFF
    |        |       |
 (360)   (#FF0000)  (red)
```

The pseudo code is as follows where `BYTE` is a byte to be mapped:

```
hue = 360 * byte / 256
region = hue / 6
offset = hue % (360 / 6)
gradient_inc = 255 * offset / (360 / 6)
gradient_dec = 255 - gradient_inc

(r, g, b) = (255, gradient_inc, 0) if region == 0
            (gradient_dec, 255, 0) if region == 1
            (0, 255, gradient_inc) if region == 2
            (0, gradient_dec, 255) if region == 3
            (gradient_inc, 0, 255) if region == 4
            (255, 0, gradient_dec) if region == 5
```

To see complete color mapping, run `picbin color-chart`.

## License

MIT
