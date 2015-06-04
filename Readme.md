Rust gpx reader [![Build Status](https://travis-ci.org/Antti/rust-gpx-reader.svg)](https://travis-ci.org/Antti/rust-gpx-reader)
===============

###Reads GuitarPro 6 compressed gpx file with BCFZ compression.
****

In order to decompress BCFZ file, you need to implement BitStream reader,
capable of reading bit-by-bit from a stream.

The format itself is pretty simple:
File has a BCFZ header following 32 le bit integer specifying expected decompressed data length.
The rest of a file consists of 2 types of data chunks (uncompressed, raw data and compressed).
Read 1 bit from a bitstream.
If bit is 0, uncompressed chunk follows:

  1. Next 2 le bits are length in bytes.
  2. Read length bytes from a bitstream.
  3. Put those bytes into a decompressed data buffer.

If bit is 1, then compressed chunk follows:

  1. Next 4 be bits are word size.
  2. Read word size bits to get offset.
  3. Read word size bits to get length.
  4. Read length bytes from offset (from the end of the current position in the decompressed data buffer).
  5. Put those bytes into a decompressed data buffer.

Sometimes there file may end before your data buffer reaches expected decompressed data lenght,
it's likely safe to assume the file was read correct.
