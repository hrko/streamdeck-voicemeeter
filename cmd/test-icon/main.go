package main

import (
	"fmt"
	"image/color"
	"image/png"
	"os"

	"github.com/hrko/streamdeck-voicemeeter/pkg/graphics"
)

func main() {
	svgFile, err := os.Create("test.svg")
	if err != nil {
		panic(err)
	}
	defer svgFile.Close()

	pngFile, err := os.Create("test.png")
	if err != nil {
		panic(err)
	}
	defer pngFile.Close()

	var p graphics.MaterialSymbolsFontParams
	p.FillEmptyWithDefault()
	// p.Style = "Rounded"

	svg, err := p.RenderIconSVG("f71a", 48, color.White, color.Black, 1)
	if err != nil {
		panic(err)
	}
	fmt.Fprint(svgFile, svg)

	img, err := p.RenderIcon("f71a", 48, color.White, color.Black, 1)
	if err != nil {
		panic(err)
	}
	png.Encode(pngFile, img)
}
