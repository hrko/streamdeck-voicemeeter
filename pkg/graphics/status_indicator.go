package graphics

import (
	"fmt"
	"image"
	"image/color"

	"github.com/fogleman/gg"
)

const (
	StatusIndicatorShapeCircle = iota
	StatusIndicatorShapeSquare
)

type StatusIndicatorShape int

type StatusIndicatorRowStyle struct {
	ColorsTrue       []color.Color
	ColorsFalse      []color.Color
	Shape            StatusIndicatorShape
	ItemMargin       float64
	ItemSize         float64
	ItemCornerRadius float64
	MarginTop        float64
	MarginLeft       float64 // only used if Rtl is false
	MarginRight      float64 // only used if Rtl is true
	Rtl              bool
}

type StatusIndicator struct {
	Width, Height int
	Rows          []StatusIndicatorRowStyle
}

func (s *StatusIndicator) Render(flags [][]bool) (image.Image, error) {
	if len(flags) > len(s.Rows) {
		return nil, fmt.Errorf("not enough styles for rows")
	}

	c := gg.NewContext(s.Width, s.Height)

	y := 0.0

	for row, rowFlags := range flags {
		style := s.Rows[row]

		if len(rowFlags) > len(style.ColorsTrue) {
			return nil, fmt.Errorf("not enough colors for row %d", row)
		}
		if len(rowFlags) > len(style.ColorsFalse) {
			return nil, fmt.Errorf("not enough inactive colors for row %d", row)
		}

		y += style.MarginTop

		for i, flag := range rowFlags {
			var (
				color color.Color
				x     float64
			)

			if flag {
				color = style.ColorsTrue[i]
			} else {
				color = style.ColorsFalse[i]
			}

			if style.Rtl {
				x = float64(s.Width) - float64(i)*(style.ItemSize+style.ItemMargin) - style.MarginRight - style.ItemSize
			} else {
				x = float64(i)*(style.ItemSize+style.ItemMargin) + style.MarginLeft
			}

			c.SetColor(color)
			switch style.Shape {
			case StatusIndicatorShapeCircle:
				c.DrawCircle(x+style.ItemSize/2, y+style.ItemSize/2, style.ItemSize/2)
			case StatusIndicatorShapeSquare:
				c.DrawRoundedRectangle(x, y, style.ItemSize, style.ItemSize, style.ItemCornerRadius)
			}
			c.Fill()
		}

		y += style.ItemSize
	}

	return c.Image(), nil
}
