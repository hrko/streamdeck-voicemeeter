package graphics

import (
	"fmt"
	"image"
	"image/color"
	"time"

	"github.com/fogleman/gg"
)

const (
	LevelMeterPeakHoldNone LevelMeterPeakHold = iota
	LevelMeterPeakHoldShowPeak
	LevelMeterPeakHoldFillPeak
	LevelMeterPeakHoldFillPeakShowCurrent
)

type LevelMeterPeakHold int

type LevelMeter struct {
	DbMin             float64
	DbGood            float64
	DbMax             float64
	PeakHold          LevelMeterPeakHold
	PeakDecayDbPerSec float64
	Image             struct {
		Width   int
		Height  int
		Padding struct {
			Top    int
			Right  int
			Bottom int
			Left   int
		}
		BackgroundColor color.Color
	}
	Cell struct {
		Length int
		Margin struct {
			X int
			Y int
		}
		Color struct {
			Normal     color.Color
			Good       color.Color
			Clipped    color.Color
			NormalOff  color.Color
			GoodOff    color.Color
			ClippedOff color.Color
		}
	}
	lastPeak struct {
		db   []float64
		time time.Time
	}
}

func (p *LevelMeter) SetDefault() {
	p.DbMin = -60.0
	p.DbGood = -24.0
	p.DbMax = 12.0
	p.Image.Width = 100
	p.Image.Height = 100
	p.Image.Padding.Top = 1
	p.Image.Padding.Right = 1
	p.Image.Padding.Bottom = 1
	p.Image.Padding.Left = 1
	p.Image.BackgroundColor = color.RGBA{R: 0x00, G: 0x00, B: 0x00, A: 0x00}
	p.Cell.Length = 1
	p.Cell.Margin.X = 1
	p.Cell.Margin.Y = 1
	p.Cell.Color.Normal = color.RGBA{R: 133, G: 173, B: 185, A: 0xff}
	p.Cell.Color.Good = color.RGBA{R: 30, G: 254, B: 91, A: 0xff}
	p.Cell.Color.Clipped = color.RGBA{R: 250, G: 0, B: 0, A: 0xff}
	p.Cell.Color.NormalOff = color.RGBA{R: 25, G: 27, B: 27, A: 0xff}
	p.Cell.Color.GoodOff = color.RGBA{R: 25, G: 27, B: 27, A: 0xff}
	p.Cell.Color.ClippedOff = color.RGBA{R: 31, G: 23, B: 21, A: 0xff}
	p.PeakHold = LevelMeterPeakHoldNone
	p.PeakDecayDbPerSec = 12.0
	p.lastPeak.db = make([]float64, 0)
	p.lastPeak.time = time.Now()
}

func (p *LevelMeter) RenderHorizontal(db []float64) (image.Image, error) {
	if p.DbMin >= p.DbMax {
		return nil, fmt.Errorf("DbMin must be less than DbMax")
	}
	if p.DbGood < p.DbMin || p.DbGood > p.DbMax {
		return nil, fmt.Errorf("DbGood must be between DbMin and DbMax")
	}
	if p.Image.Width <= 0 || p.Image.Height <= 0 {
		return nil, fmt.Errorf("Image.Width and Image.Height must be greater than 0")
	}
	if p.Image.Padding.Top < 0 || p.Image.Padding.Right < 0 || p.Image.Padding.Bottom < 0 || p.Image.Padding.Left < 0 {
		return nil, fmt.Errorf("Image.Padding values must be greater than or equal to 0")
	}
	if p.Cell.Length <= 0 {
		return nil, fmt.Errorf("Cell.Length must be greater than 0")
	}
	if p.Cell.Margin.X < 0 || p.Cell.Margin.Y < 0 {
		return nil, fmt.Errorf("Cell.Margin values must be greater than or equal to 0")
	}

	heightNoPadding := p.Image.Height - p.Image.Padding.Top - p.Image.Padding.Bottom
	widthNoPadding := p.Image.Width - p.Image.Padding.Left - p.Image.Padding.Right
	channelCount := len(db)
	cellWidth := p.Cell.Length
	cellHeight := (heightNoPadding - p.Cell.Margin.Y*(channelCount-1)) / channelCount
	cellCount := (widthNoPadding + p.Cell.Margin.X) / (cellWidth + p.Cell.Margin.X)
	minGoodCellIndex := int((p.DbGood - p.DbMin) / (p.DbMax - p.DbMin) * float64(cellCount))
	minClipCellIndex := int((0.0 - p.DbMin) / (p.DbMax - p.DbMin) * float64(cellCount))
	if cellHeight == 0 {
		return nil, fmt.Errorf("calculated cellHeight is 0")
	}
	if cellCount == 0 {
		return nil, fmt.Errorf("calculated cellCount is 0")
	}

	peak := p.lastPeak.db
	if len(peak) != channelCount {
		peak = make([]float64, channelCount)
		for i := 0; i < channelCount; i++ {
			peak[i] = -200.0
		}
	}
	peakDecay := p.PeakDecayDbPerSec * time.Since(p.lastPeak.time).Seconds()
	for i := 0; i < channelCount; i++ {
		if db[i] > peak[i] {
			peak[i] = db[i]
		} else {
			peak[i] -= peakDecay
		}
	}
	p.lastPeak.db = peak
	p.lastPeak.time = time.Now()
	cellIndexPeak := make([]int, channelCount)
	for i := 0; i < channelCount; i++ {
		cellIndexPeak[i] = int((peak[i] - p.DbMin) / (p.DbMax - p.DbMin) * float64(cellCount))
	}

	img := image.NewRGBA(image.Rect(0, 0, p.Image.Width, p.Image.Height))
	dc := gg.NewContextForRGBA(img)

	dc.SetColor(p.Image.BackgroundColor)
	dc.DrawRectangle(0, 0, float64(p.Image.Width), float64(p.Image.Height))
	dc.Fill()

	for ch, lvDb := range db {
		// normalize lvDb to 0.0-1.0
		lv := 0.0
		if lvDb > p.DbMax {
			lv = 1.0
		} else if lvDb > p.DbMin {
			lv = (lvDb - p.DbMin) / (p.DbMax - p.DbMin)
		} else {
			lv = 0.0
		}
		cellIndexCurrent := int(lv * float64(cellCount))

		// calculate minOffCellIndex
		var minOffCellIndex int
		switch p.PeakHold {
		case LevelMeterPeakHoldNone, LevelMeterPeakHoldShowPeak:
			minOffCellIndex = cellIndexCurrent
		case LevelMeterPeakHoldFillPeak, LevelMeterPeakHoldFillPeakShowCurrent:
			minOffCellIndex = cellIndexPeak[ch]
		}

		// draw cells
		for i := 0; i < cellCount; i++ {
			x := i*(cellWidth+p.Cell.Margin.X) + p.Image.Padding.Left
			y := ch*(cellHeight+p.Cell.Margin.Y) + p.Image.Padding.Top
			w := cellWidth
			h := cellHeight
			if i < minOffCellIndex {
				if i < minGoodCellIndex {
					dc.SetColor(p.Cell.Color.Normal)
				} else if i < minClipCellIndex {
					dc.SetColor(p.Cell.Color.Good)
				} else {
					dc.SetColor(p.Cell.Color.Clipped)
				}
			} else {
				if i < minGoodCellIndex {
					dc.SetColor(p.Cell.Color.NormalOff)
				} else if i < minClipCellIndex {
					dc.SetColor(p.Cell.Color.GoodOff)
				} else {
					dc.SetColor(p.Cell.Color.ClippedOff)
				}
			}
			dc.DrawRectangle(float64(x), float64(y), float64(w), float64(h))
			dc.Fill()
		}

		// draw peak or current level indicator
		switch p.PeakHold {
		case LevelMeterPeakHoldShowPeak:
			index := cellIndexPeak[ch]
			if index < minGoodCellIndex {
				dc.SetColor(p.Cell.Color.Normal)
			} else if index < minClipCellIndex {
				dc.SetColor(p.Cell.Color.Good)
			} else {
				dc.SetColor(p.Cell.Color.Clipped)
			}
			x := index*(cellWidth+p.Cell.Margin.X) + p.Image.Padding.Left
			y := ch*(cellHeight+p.Cell.Margin.Y) + p.Image.Padding.Top
			w := cellWidth
			h := cellHeight
			dc.DrawRectangle(float64(x), float64(y), float64(w), float64(h))
			dc.Fill()
		case LevelMeterPeakHoldFillPeakShowCurrent:
			index := cellIndexCurrent
			if index != cellIndexPeak[ch] {
				if index < minGoodCellIndex {
					dc.SetColor(p.Cell.Color.NormalOff)
				} else if index < minClipCellIndex {
					dc.SetColor(p.Cell.Color.GoodOff)
				} else {
					dc.SetColor(p.Cell.Color.ClippedOff)
				}
				x := index*(cellWidth+p.Cell.Margin.X) + p.Image.Padding.Left
				y := ch*(cellHeight+p.Cell.Margin.Y) + p.Image.Padding.Top
				w := cellWidth
				h := cellHeight
				dc.DrawRectangle(float64(x), float64(y), float64(w), float64(h))
				dc.Fill()
			}
		}
	}

	return img, nil
}
