# Volym Utils

Strips the segments and binary data and out of a segmentation file generated by
3D Slicer. 

## Run

```bash
cargo run -- ./Segmentation_1.seg.nrrd segments.json segments.raw
```

For importance based volume rendering, the generated json should be edited, to
specify the desired importance of each segment (default: 0).
