#!/usr/bin/env python3

# This very primitive script takes an image input and produces a C header file.
# We make some pretty bold assumptions, like the font is 12x12, has 256 glyphs, image is 192x192, etc.
# In the future, this should be fixed, and font info standardized.

import sys

from PIL import Image

atlas_width = 16
atlas_height = 16
glyph_width = 12
glyph_height = 12

for infile in sys.argv[1:]:
	try:
		with Image.open(infile) as im:
			#print(im.size)

			if not (im.size[0] == glyph_width * atlas_width and im.size[1] == glyph_height * atlas_height):
				print(f"Image Error! Font image must be {glyph_width * atlas_height} x {glyph_height * atlas_height}!")
				continue

			im = im.convert("1")

			atlas = []
			for ay in range(atlas_height):
				for ax in range(atlas_width):
					glyph = []
					for row in range(glyph_height):
						prow = 0
						for col in range(glyph_width):
							p = im.getpixel((col + ax * 12, row + ay * 12))
							if p > 0:
								prow |= 1<<(glyph_width-1-col)
						#print(f"{prow:0{glyph_width}b}")
						glyph.append(prow)
					#print()
					atlas.append(glyph)
			#print(atlas)

			header = ""
			header += "#include <pico/stdlib.h>\n"
			header += "\n"
			header += "#ifndef JUKEBOX_FONT_H\n"
			header += "#define JUKEBOX_FONT_H\n"
			header += "\n"
			header += f"const uint8_t glyph_width = {glyph_width};\n"
			header += f"const uint8_t glyph_height = {glyph_height};\n"
			header += "\n"
			header += f"const uint16_t font[256][{glyph_height}] = " + "{\n"
			for glyph in atlas:
				header += "\t{ "
				for row in glyph:
					header += f"0x{row:04x}, "
				header += "}, \n"
			header += "};\n"
			header += "\n"
			header += "#endif // JUKEBOX_FONT_H\n"

			print(header)

			pass # TODO: iterate over image and process bits
	except OSError as e:
		print("OS Error!", e, infile)
