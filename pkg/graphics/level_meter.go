package graphics

import (
	"fmt"
	"image"
	"image/color"
	"math"
	"time"

	"github.com/disintegration/imaging"
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
	ChannelCount      int
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
	p.lastPeak.db = make([]float64, p.ChannelCount)
	for i := range p.lastPeak.db {
		p.lastPeak.db[i] = -200.0
	}
	p.lastPeak.time = time.Now()
}

func (p *LevelMeter) RenderHorizontal(db []float64) (image.Image, error) {
	if len(db) < p.ChannelCount {
		return nil, fmt.Errorf("db length is less than ChannelCount")
	}

	err := p.validateConfig()
	if err != nil {
		return nil, err
	}

	cellWidth := p.Cell.Length
	cellHeight := p.calculateCellHeight()
	cellCount := p.calculateCellCount()
	if cellHeight == 0 {
		return nil, fmt.Errorf("calculated cellHeight is 0")
	}
	if cellCount == 0 {
		return nil, fmt.Errorf("calculated cellCount is 0")
	}

	peak := p.updatePeak(db)

	img := image.NewRGBA(image.Rect(0, 0, p.Image.Width, p.Image.Height))
	dc := gg.NewContextForRGBA(img)
	dc.SetColor(p.Image.BackgroundColor)
	bgWidth := p.Image.Padding.Left + p.Image.Padding.Right + cellCount*(cellWidth+p.Cell.Margin.X) - p.Cell.Margin.X
	bgHeight := p.Image.Padding.Top + p.Image.Padding.Bottom + p.ChannelCount*(cellHeight+p.Cell.Margin.Y) - p.Cell.Margin.Y
	dc.DrawRectangle(0, 0, float64(bgWidth), float64(bgHeight))
	dc.Fill()

	for ch, currentLv := range db {
		cellIndexCurrent := p.calculateCellIndex(currentLv)
		cellIndexPeak := p.calculateCellIndex(peak[ch])

		var minOffCellIndex int
		switch p.PeakHold {
		case LevelMeterPeakHoldNone, LevelMeterPeakHoldShowPeak:
			minOffCellIndex = cellIndexCurrent
		case LevelMeterPeakHoldFillPeak, LevelMeterPeakHoldFillPeakShowCurrent:
			minOffCellIndex = cellIndexPeak
		}

		// draw cells
		for i := 0; i < cellCount; i++ {
			if i < minOffCellIndex {
				dc.SetColor(p.getCellColor(i))
			} else {
				dc.SetColor(p.getCellColorOff(i))
			}
			p.drawAndFillCell(dc, ch, i, cellWidth, cellHeight)
		}

		// draw peak or current level indicator cell
		switch p.PeakHold {
		case LevelMeterPeakHoldShowPeak:
			i := cellIndexPeak
			dc.SetColor(p.getCellColor(i))
			p.drawAndFillCell(dc, ch, i, cellWidth, cellHeight)
		case LevelMeterPeakHoldFillPeakShowCurrent:
			i := cellIndexCurrent
			if i != cellIndexPeak {
				dc.SetColor(p.getCellColorOff(i))
				p.drawAndFillCell(dc, ch, i, cellWidth, cellHeight)
			}
		}
	}

	return img, nil
}

// render with RenderHorizontal and rotate 90 degrees
func (p *LevelMeter) RenderVertical(db []float64) (image.Image, error) {
	copy := *p
	copy.Image.Width = p.Image.Height
	copy.Image.Height = p.Image.Width
	copy.Image.Padding.Top = p.Image.Padding.Right
	copy.Image.Padding.Right = p.Image.Padding.Bottom
	copy.Image.Padding.Bottom = p.Image.Padding.Left
	copy.Image.Padding.Left = p.Image.Padding.Top
	copy.Cell.Margin.X = p.Cell.Margin.Y
	copy.Cell.Margin.Y = p.Cell.Margin.X

	img, err := copy.RenderHorizontal(db)
	if err != nil {
		return nil, err
	}

	p.lastPeak = copy.lastPeak

	return imaging.Rotate90(img), nil
}

func (p *LevelMeter) validateConfig() error {
	if p.DbMin >= p.DbMax {
		return fmt.Errorf("DbMin must be less than DbMax")
	}
	if p.DbGood < p.DbMin || p.DbGood > p.DbMax {
		return fmt.Errorf("DbGood must be between DbMin and DbMax")
	}
	if p.DbMax < 0.0 {
		return fmt.Errorf("DbMax must be greater than 0.0")
	}
	if p.Image.Width <= 0 || p.Image.Height <= 0 {
		return fmt.Errorf("Image.Width and Image.Height must be greater than 0")
	}
	if p.Image.Padding.Top < 0 || p.Image.Padding.Right < 0 || p.Image.Padding.Bottom < 0 || p.Image.Padding.Left < 0 {
		return fmt.Errorf("Image.Padding values must be greater than or equal to 0")
	}
	if p.Cell.Length <= 0 {
		return fmt.Errorf("Cell.Length must be greater than 0")
	}
	if p.Cell.Margin.X < 0 || p.Cell.Margin.Y < 0 {
		return fmt.Errorf("Cell.Margin values must be greater than or equal to 0")
	}
	return nil
}

func (p *LevelMeter) calculateCellCount() int {
	widthNoPadding := p.Image.Width - p.Image.Padding.Left - p.Image.Padding.Right
	cellCount := (widthNoPadding + p.Cell.Margin.X) / (p.Cell.Length + p.Cell.Margin.X)
	return cellCount
}

func (p *LevelMeter) calculateCellHeight() int {
	heightNoPadding := p.Image.Height - p.Image.Padding.Top - p.Image.Padding.Bottom
	cellHeight := (heightNoPadding - p.Cell.Margin.Y*(p.ChannelCount-1)) / p.ChannelCount
	return cellHeight
}

func (p *LevelMeter) updatePeak(db []float64) []float64 {
	elapsed := time.Since(p.lastPeak.time)
	decay := p.PeakDecayDbPerSec * elapsed.Seconds()
	for ch, currentLv := range db {
		if currentLv > p.lastPeak.db[ch] {
			p.lastPeak.db[ch] = currentLv
		} else {
			p.lastPeak.db[ch] -= decay
		}
	}
	p.lastPeak.time = time.Now()
	return p.lastPeak.db
}

func (p *LevelMeter) calculateCellIndex(lvDb float64) int {
	cellCount := p.calculateCellCount()
	cellIndex := int(math.Round((lvDb - p.DbMin) / (p.DbMax - p.DbMin) * float64(cellCount)))
	return cellIndex
}

func (p *LevelMeter) getCellColor(cellIndex int) color.Color {
	minGoodCellIndex := p.calculateCellIndex(p.DbGood)
	minClipCellIndex := p.calculateCellIndex(0.0)
	if cellIndex < minGoodCellIndex {
		return p.Cell.Color.Normal
	} else if cellIndex < minClipCellIndex {
		return p.Cell.Color.Good
	} else {
		return p.Cell.Color.Clipped
	}
}

func (p *LevelMeter) getCellColorOff(cellIndex int) color.Color {
	minGoodCellIndex := p.calculateCellIndex(p.DbGood)
	minClipCellIndex := p.calculateCellIndex(0.0)
	if cellIndex < minGoodCellIndex {
		return p.Cell.Color.NormalOff
	} else if cellIndex < minClipCellIndex {
		return p.Cell.Color.GoodOff
	} else {
		return p.Cell.Color.ClippedOff
	}
}

func (p *LevelMeter) drawAndFillCell(dc *gg.Context, ch, cellIndex, cellWidth, cellHeight int) {
	if ch >= p.ChannelCount || ch < 0 {
		return
	}
	if cellIndex >= p.calculateCellCount() || cellIndex < 0 {
		return
	}
	x := cellIndex*(cellWidth+p.Cell.Margin.X) + p.Image.Padding.Left
	y := ch*(cellHeight+p.Cell.Margin.Y) + p.Image.Padding.Top
	w := cellWidth
	h := cellHeight
	dc.DrawRectangle(float64(x), float64(y), float64(w), float64(h))
	dc.Fill()
}
