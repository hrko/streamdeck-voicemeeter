package graphics

import (
	"image"
	"image/color"

	"github.com/fogleman/gg"
)

type GainFader struct {
	Color struct {
		Background              color.Color
		Boarder                 color.Color
		ForegroundNormal        color.Color
		ForegroundOveramplified color.Color
	}
	Width                    int
	Height                   int
	RoundedCorners           bool
	BoarderWidth             int
	BoarderColorIsForeground bool
	DbMin                    float64
	DbMax                    float64
}

func NewGainFader() *GainFader {
	g := &GainFader{}
	g.Color.Background = color.RGBA{0x2c, 0x3d, 0x4d, 0xff}
	g.Color.Boarder = color.RGBA{0, 0, 0, 159}
	g.Color.ForegroundNormal = color.RGBA{0x70, 0xc3, 0x99, 0xff}
	g.Color.ForegroundOveramplified = color.RGBA{0xf8, 0x63, 0x4d, 0xff}
	g.Width = 108
	g.Height = 12
	g.RoundedCorners = true
	g.BoarderWidth = 2
	g.BoarderColorIsForeground = false
	g.DbMin = -60.0
	g.DbMax = 12.0
	return g
}

func (g *GainFader) RenderHorizontal(db float64) image.Image {
	c := gg.NewContext(g.Width, g.Height)

	fgColor := g.Color.ForegroundNormal
	if db > 0.0 {
		fgColor = g.Color.ForegroundOveramplified
	}

	// define drawing area
	w := float64(g.Width)
	h := float64(g.Height)
	r := min(w, h) / 2
	if !g.RoundedCorners {
		r = 0.0
	}
	c.DrawRoundedRectangle(0, 0, w, h, r)
	c.Clip()

	// draw boarder
	c.DrawRectangle(0, 0, w, h)
	c.SetColor(g.Color.Boarder)
	c.Fill()

	// paint background
	x := float64(g.BoarderWidth)
	y := float64(g.BoarderWidth)
	w = float64(g.Width - g.BoarderWidth*2)
	h = float64(g.Height - g.BoarderWidth*2)
	r = min(w, h) / 2
	if !g.RoundedCorners {
		r = 0.0
	}
	c.DrawRoundedRectangle(x, y, w, h, r)
	c.SetColor(g.Color.Background)
	c.Fill()

	if db == g.DbMin {
		return c.Image()
	}

	// paint foreground
	x = float64(g.BoarderWidth)
	y = float64(g.BoarderWidth)
	w = float64(g.Width-g.BoarderWidth*2) * (db - g.DbMin) / (g.DbMax - g.DbMin)
	h = float64(g.Height - g.BoarderWidth*2)
	r = min(w, h) / 2
	if !g.RoundedCorners {
		r = 0.0
	}
	c.DrawRoundedRectangle(x, y, w, h, r)
	c.SetColor(fgColor)
	c.Fill()

	return c.Image()
}
