# xgcode

`xgcode` is a proprietary file format used by FlashForge 3D printers.

It wraps a plain g-code text file with a binary header containing some information
about the build (time, filament consumption, temperature settings...) and a bitmap
thumbnail. This extra information is used by the printer's integrated interface.

This library is for parsing and generating xgcode header, and accessing the embedded
BMP thumbnail and gcode payload.