silence.flac is a self-generated, public-domain test fixture: 128 fixed-size
4096-sample constant-zero frames, stereo 16-bit 48 kHz (524,288 samples/channel).
It contains only STREAMINFO and constant subframes, with frame CRC-8/CRC-16;
the optional audio MD5 is unspecified (all zero). No station audio is included.

The loopback tests also wrap these frames in native FLAC comments and chained
Ogg-FLAC pages with generated CRCs, titles and station fields. This exercises the
production demuxer/decoder and container metadata revisions without radio audio.
