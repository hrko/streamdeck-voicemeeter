package gain_controll_combo

// TODO:
// - [ ] Separate the code common to "gain_controll" into a separate package.
// - [ ] Refactor render() since it's too long.

import (
	"context"
	"encoding/json"
	"fmt"
	"image/color"
	"log"
	"strconv"
	"time"

	"github.com/fufuok/cmap"
	"github.com/hrko/streamdeck"
	sdcontext "github.com/hrko/streamdeck/context"
	"github.com/onyx-and-iris/voicemeeter/v2"

	"github.com/hrko/streamdeck-voicemeeter/internal/stripbus"
	"github.com/hrko/streamdeck-voicemeeter/pkg/graphics"
)

const (
	ActionUUID = "jp.hrko.streamdeck.voicemeeter.gain-controll-combo"
)

var (
	instanceMap    *cmap.MapOf[string, instanceProperty]
	renderCh       chan *renderParams
	levelMeterMap  *cmap.MapOf[string, *graphics.LevelMeter]
	levelMeter1Map *cmap.MapOf[string, *graphics.LevelMeter]
)

type instanceProperty streamdeck.WillAppearPayload[instanceSettings]

type instanceSettings struct {
	IconCodePoint    string                             `json:"iconCodePoint,omitempty"`
	IconFontParams   graphics.MaterialSymbolsFontParams `json:"iconFontParams,omitempty"`
	StripOrBusKind   string                             `json:"stripOrBusKind,omitempty"` // "Strip" | "Bus"
	StripOrBusIndex  int                                `json:"stripOrBusIndex,omitempty"`
	GainDelta        string                             `json:"gainDelta,omitempty"`
	IconCodePoint1   string                             `json:"iconCodePoint1,omitempty"`
	IconFontParams1  graphics.MaterialSymbolsFontParams `json:"iconFontParams1,omitempty"`
	StripOrBusKind1  string                             `json:"stripOrBusKind1,omitempty"` // "Strip" | "Bus"
	StripOrBusIndex1 int                                `json:"stripOrBusIndex1,omitempty"`
	GainDelta1       string                             `json:"gainDelta1,omitempty"`
}

type renderParams struct {
	targetContext string
	title         *string
	settings      *instanceSettings
	levels        *[]float64
	gain          *float64
	status        stripbus.IStripOrBusStatus
	title1        *string
	levels1       *[]float64
	gain1         *float64
	status1       stripbus.IStripOrBusStatus
}

func defaultInstanceSettings() instanceSettings {
	return instanceSettings{
		IconCodePoint: "",
		IconFontParams: graphics.MaterialSymbolsFontParams{
			Style: "Rounded",
			Opsz:  "20",
			Wght:  "400",
			Fill:  "0",
			Grad:  "0",
		},
		StripOrBusKind:  "Strip",
		StripOrBusIndex: 0,
		GainDelta:       "3.0",
		IconCodePoint1:  "",
		IconFontParams1: graphics.MaterialSymbolsFontParams{
			Style: "Rounded",
			Opsz:  "20",
			Wght:  "400",
			Fill:  "0",
			Grad:  "0",
		},
		StripOrBusKind1:  "Bus",
		StripOrBusIndex1: 0,
		GainDelta1:       "3.0",
	}
}

func SetupPreClientRun(client *streamdeck.Client) {
	action := client.Action(ActionUUID)
	instanceMap = cmap.NewOf[string, instanceProperty]() // key: context of action instance
	renderCh = make(chan *renderParams, 32)

	action.RegisterHandler(streamdeck.DidReceiveSettings, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.WillAppearPayload[instanceSettings]
		p.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

		if instanceMap.Has(event.Context) {
			var dummy instanceProperty
			instanceMap.Upsert(event.Context, dummy, func(exist bool, valueInMap, _ instanceProperty) instanceProperty {
				valueInMap.Settings = p.Settings
				return valueInMap
			})
		}

		renderCh <- &renderParams{
			targetContext: event.Context,
			settings:      &p.Settings,
		}
		return nil
	})

	action.RegisterHandler(streamdeck.WillAppear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.WillAppearPayload[instanceSettings]
		p.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		instanceMap.Set(event.Context, instanceProperty(p))
		renderCh <- &renderParams{
			targetContext: event.Context,
			settings:      &p.Settings,
		}
		return nil
	})

	action.RegisterHandler(streamdeck.WillDisappear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		instanceMap.Remove(event.Context)
		return nil
	})
}

func SetupPostClientRun(client *streamdeck.Client, vm *voicemeeter.Remote) error {
	action := client.Action(ActionUUID)
	levelMeterMap = cmap.NewOf[string, *graphics.LevelMeter]()  // key: context of action instance
	levelMeter1Map = cmap.NewOf[string, *graphics.LevelMeter]() // key: context of action instance

	action.RegisterHandler(streamdeck.DialRotate, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.DialRotatePayload[instanceSettings]
		p.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

		gainDelta, err := strconv.ParseFloat(p.Settings.GainDelta, 64)
		if err != nil {
			log.Printf("error parsing gainDelta: %v\n", err)
			gainDelta = 3.0 // default
		}
		if p.Pressed {
			switch p.Settings.StripOrBusKind1 {
			case "Strip":
				adjustStripGain(vm, p.Settings.StripOrBusIndex1, gainDelta*float64(p.Ticks))
			case "Bus":
				adjustBusGain(vm, p.Settings.StripOrBusIndex1, gainDelta*float64(p.Ticks))
			default:
				log.Printf("unknown stripOrBusKind1: '%v'\n", p.Settings.StripOrBusKind1)
			}
			renderParams := newRenderParams(event.Context)
			renderParams.SetGain1(vm, p.Settings.StripOrBusKind1, p.Settings.StripOrBusIndex1)
			renderCh <- renderParams
		} else {
			switch p.Settings.StripOrBusKind {
			case "Strip":
				adjustStripGain(vm, p.Settings.StripOrBusIndex, gainDelta*float64(p.Ticks))
			case "Bus":
				adjustBusGain(vm, p.Settings.StripOrBusIndex, gainDelta*float64(p.Ticks))
			default:
				log.Printf("unknown stripOrBusKind: '%v'\n", p.Settings.StripOrBusKind)
			}
			renderParams := newRenderParams(event.Context)
			renderParams.SetGain(vm, p.Settings.StripOrBusKind, p.Settings.StripOrBusIndex)
			renderCh <- renderParams
		}

		return nil
	})

	action.RegisterHandler(streamdeck.TouchTap, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.TouchTapPayload[instanceSettings]
		p.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

		const touchPadWidth = 200
		posX := p.TapPos[0]
		if posX < touchPadWidth/2 {
			switch p.Settings.StripOrBusKind {
			case "Strip":
				toggleMuteStrip(vm, p.Settings.StripOrBusIndex)
				if err != nil {
					log.Printf("error toggling mute: %v\n", err)
				}
			case "Bus":
				toggleMuteBus(vm, p.Settings.StripOrBusIndex)
				if err != nil {
					log.Printf("error toggling mute: %v\n", err)
				}
			default:
				log.Printf("unknown stripOrBusKind: '%v'\n", p.Settings.StripOrBusKind)
			}
			renderParams := newRenderParams(event.Context)
			renderParams.SetStatus(vm, p.Settings.StripOrBusKind, p.Settings.StripOrBusIndex)
			renderCh <- renderParams
		} else {
			switch p.Settings.StripOrBusKind1 {
			case "Strip":
				toggleMuteStrip(vm, p.Settings.StripOrBusIndex1)
				if err != nil {
					log.Printf("error toggling mute: %v\n", err)
				}
			case "Bus":
				toggleMuteBus(vm, p.Settings.StripOrBusIndex1)
				if err != nil {
					log.Printf("error toggling mute: %v\n", err)
				}
			default:
				log.Printf("unknown stripOrBusKind: '%v'\n", p.Settings.StripOrBusKind1)
			}
			renderParams := newRenderParams(event.Context)
			renderParams.SetStatus1(vm, p.Settings.StripOrBusKind1, p.Settings.StripOrBusIndex1)
			renderCh <- renderParams
		}

		return nil
	})

	go func() {
		for renderParam := range renderCh {
			render(client, renderParam)
		}
	}()

	go func() {
		const refreshInterval = time.Second / 15
		for range time.Tick(refreshInterval) {
			for item := range instanceMap.IterBuffered() {
				actionContext := item.Key
				actionProps := item.Val
				go func() {
					renderParam := newRenderParams(actionContext)

					stripOrBusKind := actionProps.Settings.StripOrBusKind
					stripOrBusIndex := actionProps.Settings.StripOrBusIndex
					stripOrBusKind1 := actionProps.Settings.StripOrBusKind1
					stripOrBusIndex1 := actionProps.Settings.StripOrBusIndex1

					renderParam.SetTitle(vm, stripOrBusKind, stripOrBusIndex)
					renderParam.SetLevels(vm, stripOrBusKind, stripOrBusIndex)
					renderParam.SetGain(vm, stripOrBusKind, stripOrBusIndex)
					renderParam.SetStatus(vm, stripOrBusKind, stripOrBusIndex)
					renderParam.SetTitle1(vm, stripOrBusKind1, stripOrBusIndex1)
					renderParam.SetLevels1(vm, stripOrBusKind1, stripOrBusIndex1)
					renderParam.SetGain1(vm, stripOrBusKind1, stripOrBusIndex1)
					renderParam.SetStatus1(vm, stripOrBusKind1, stripOrBusIndex1)

					renderCh <- renderParam
				}()
			}
		}
	}()

	return nil
}

func newRenderParams(actionContext string) *renderParams {
	return &renderParams{
		targetContext: actionContext,
	}
}

func (p *renderParams) SetLevels(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	l, err := getLevels(vm, stripOrBusKind, stripOrBusIndex)
	if err != nil {
		log.Printf("error getting levels: %v\n", err)
		return
	}
	p.levels = &l
}

func (p *renderParams) SetLevels1(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	l, err := getLevels(vm, stripOrBusKind, stripOrBusIndex)
	if err != nil {
		log.Printf("error getting levels: %v\n", err)
		return
	}
	p.levels1 = &l
}

func getLevels(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) ([]float64, error) {
	switch stripOrBusKind {
	case "Strip":
		stripCount := len(vm.Strip)
		if stripOrBusIndex >= stripCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return nil, fmt.Errorf("stripOrBusIndex %v is out of range", stripOrBusIndex)
		}
		levels := vm.Strip[stripOrBusIndex].Levels().PostFader()
		levels = levels[:2]
		return levels, nil

	case "Bus":
		busCount := len(vm.Bus)
		if stripOrBusIndex >= busCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return nil, fmt.Errorf("stripOrBusIndex %v is out of range", stripOrBusIndex)
		}
		levels := vm.Bus[stripOrBusIndex].Levels().All()
		levels = levels[:2]
		return levels, nil

	default:
		log.Printf("unknown stripOrBusKind: '%v'\n", stripOrBusKind)
		return nil, fmt.Errorf("unknown stripOrBusKind: '%v'", stripOrBusKind)
	}
}

func (p *renderParams) SetTitle(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	title, err := getTitle(vm, stripOrBusKind, stripOrBusIndex)
	if err != nil {
		log.Printf("error getting title: %v\n", err)
		return
	}
	p.title = &title
}

func (p *renderParams) SetTitle1(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	title, err := getTitle(vm, stripOrBusKind, stripOrBusIndex)
	if err != nil {
		log.Printf("error getting title: %v\n", err)
		return
	}
	p.title1 = &title
}

func getTitle(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) (string, error) {
	switch stripOrBusKind {
	case "Strip":
		stripCount := len(vm.Strip)
		if stripOrBusIndex >= stripCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return "", fmt.Errorf("stripOrBusIndex %v is out of range", stripOrBusIndex)
		}
		title := vm.Strip[stripOrBusIndex].Label()
		if title == "" {
			title = fmt.Sprintf("Strip %v", stripOrBusIndex)
		}
		return title, nil

	case "Bus":
		busCount := len(vm.Bus)
		if stripOrBusIndex >= busCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return "", fmt.Errorf("stripOrBusIndex %v is out of range", stripOrBusIndex)
		}
		title := vm.Bus[stripOrBusIndex].Label()
		if title == "" {
			title = fmt.Sprintf("Bus %v", stripOrBusIndex)
		}
		return title, nil

	default:
		log.Printf("unknown stripOrBusKind: '%v'\n", stripOrBusKind)
		return "", fmt.Errorf("unknown stripOrBusKind: '%v'", stripOrBusKind)
	}
}

func (p *renderParams) SetGain(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	gain, err := getGain(vm, stripOrBusKind, stripOrBusIndex)
	if err != nil {
		log.Printf("error getting gain: %v\n", err)
		return
	}
	p.gain = &gain
}

func (p *renderParams) SetGain1(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	gain, err := getGain(vm, stripOrBusKind, stripOrBusIndex)
	if err != nil {
		log.Printf("error getting gain: %v\n", err)
		return
	}
	p.gain1 = &gain
}

func getGain(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) (float64, error) {
	switch stripOrBusKind {
	case "Strip":
		stripCount := len(vm.Strip)
		if stripOrBusIndex >= stripCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return 0, fmt.Errorf("stripOrBusIndex %v is out of range", stripOrBusIndex)
		}
		gain := vm.Strip[stripOrBusIndex].Gain()
		return gain, nil

	case "Bus":
		busCount := len(vm.Bus)
		if stripOrBusIndex >= busCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return 0, fmt.Errorf("stripOrBusIndex %v is out of range", stripOrBusIndex)
		}
		gain := vm.Bus[stripOrBusIndex].Gain()
		return gain, nil

	default:
		log.Printf("unknown stripOrBusKind: '%v'\n", stripOrBusKind)
		return 0, fmt.Errorf("unknown stripOrBusKind: '%v'", stripOrBusKind)
	}
}

func (p *renderParams) SetStatus(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	s, err := stripbus.GetStripOrBusStatus(vm, stripOrBusKind, stripOrBusIndex)
	if err != nil {
		log.Printf("error getting strip or bus status: %v\n", err)
		return
	}
	p.status = s
}

func (p *renderParams) SetStatus1(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	s, err := stripbus.GetStripOrBusStatus(vm, stripOrBusKind, stripOrBusIndex)
	if err != nil {
		log.Printf("error getting strip or bus status: %v\n", err)
		return
	}
	p.status1 = s
}

func render(client *streamdeck.Client, renderParam *renderParams) error {
	ctx := context.Background()
	ctx = sdcontext.WithContext(ctx, renderParam.targetContext)

	instProps, ok := instanceMap.Get(renderParam.targetContext)
	if !ok {
		return fmt.Errorf("instanceMap has no key '%v'", renderParam.targetContext)
	}

	levelMeter, ok := levelMeterMap.Get(renderParam.targetContext)
	if !ok {
		levelMeter = graphics.NewLevelMeter(2)
		levelMeterMap.Set(renderParam.targetContext, levelMeter)
	}
	levelMeter1, ok := levelMeter1Map.Get(renderParam.targetContext)
	if !ok {
		levelMeter1 = graphics.NewLevelMeter(2)
		levelMeter1Map.Set(renderParam.targetContext, levelMeter1)
	}

	switch instProps.Controller {
	case "Encoder":
		payload := struct {
			Title       *string `json:"title,omitempty"`
			Icon        *string `json:"icon,omitempty"`
			LevelMeter  *string `json:"levelMeter,omitempty"`
			GainValue   *string `json:"gainValue,omitempty"`
			GainSlider  *string `json:"gainSlider,omitempty"`
			Status      *string `json:"status,omitempty"`
			Title1      *string `json:"title1,omitempty"`
			Icon1       *string `json:"icon1,omitempty"`
			LevelMeter1 *string `json:"levelMeter1,omitempty"`
			GainValue1  *string `json:"gainValue1,omitempty"`
			GainSlider1 *string `json:"gainSlider1,omitempty"`
			Status1     *string `json:"status1,omitempty"`
		}{}

		if renderParam.title != nil {
			payload.Title = renderParam.title
		}
		if renderParam.title1 != nil {
			payload.Title1 = renderParam.title1
		}
		if renderParam.settings != nil {
			fontParams := renderParam.settings.IconFontParams
			if err := fontParams.Assert(); err != nil {
				log.Printf("invalid iconFontParams: %v\n", err)
				fontParams = graphics.MaterialSymbolsFontParams{}
				fontParams.FillEmptyWithDefault()
			}
			iconCodePoint := renderParam.settings.IconCodePoint
			if iconCodePoint == "" {
				switch renderParam.settings.StripOrBusKind {
				case "Strip", "":
					iconCodePoint = "f71a" // input_circle
				case "Bus":
					iconCodePoint = "f70e" // output_circle
				}
			}
			img, err := fontParams.RenderIcon(iconCodePoint, 20, color.White, color.RGBA{0, 0, 0, 120}, 1)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error converting image to base64: %v\n", err)
			}
			payload.Icon = &imgBase64
		}
		if renderParam.settings != nil {
			fontParams := renderParam.settings.IconFontParams1
			if err := fontParams.Assert(); err != nil {
				log.Printf("invalid iconFontParams: %v\n", err)
				fontParams = graphics.MaterialSymbolsFontParams{}
				fontParams.FillEmptyWithDefault()
			}
			iconCodePoint := renderParam.settings.IconCodePoint1
			if iconCodePoint == "" {
				switch renderParam.settings.StripOrBusKind1 {
				case "Strip", "":
					iconCodePoint = "f71a" // input_circle
				case "Bus":
					iconCodePoint = "f70e" // output_circle
				}
			}
			img, err := fontParams.RenderIcon(iconCodePoint, 20, color.White, color.RGBA{0, 0, 0, 128}, 1)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error converting image to base64: %v\n", err)
			}
			payload.Icon1 = &imgBase64
		}
		if renderParam.levels != nil {
			levelMeter.Image.Width = 84
			levelMeter.Image.Height = 5
			levelMeter.Image.Padding.Left = 2
			levelMeter.Image.Padding.Right = 1
			levelMeter.Cell.Length = 1
			levelMeter.PeakHold = graphics.LevelMeterPeakHoldFillPeakShowCurrent
			img, err := levelMeter.RenderHorizontal(*renderParam.levels)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			payload.LevelMeter = &imgBase64
		}
		if renderParam.levels1 != nil {
			levelMeter1.Image.Width = 84
			levelMeter1.Image.Height = 5
			levelMeter1.Image.Padding.Left = 2
			levelMeter1.Image.Padding.Right = 1
			levelMeter1.Cell.Length = 1
			levelMeter1.PeakHold = graphics.LevelMeterPeakHoldFillPeakShowCurrent
			img, err := levelMeter1.RenderHorizontal(*renderParam.levels1)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			payload.LevelMeter1 = &imgBase64
		}
		if renderParam.gain != nil {
			str := fmt.Sprintf("%.1f", *renderParam.gain)
			payload.GainValue = &str

			gainFader := graphics.NewGainFader()
			gainFader.Width = 84
			gainFader.Height = 12
			img := gainFader.RenderHorizontal(*renderParam.gain)
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			payload.GainSlider = &imgBase64
		}
		if renderParam.gain1 != nil {
			str := fmt.Sprintf("%.1f", *renderParam.gain1)
			payload.GainValue1 = &str

			gainFader := graphics.NewGainFader()
			gainFader.Width = 84
			gainFader.Height = 12
			img := gainFader.RenderHorizontal(*renderParam.gain1)
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			payload.GainSlider1 = &imgBase64
		}
		if renderParam.status != nil {
			s := renderParam.status
			img, err := s.RenderIndicator()
			if err != nil {
				log.Printf("error creating image: %v\n", err)
			}
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
			}
			payload.Status = &imgBase64
		}
		if renderParam.status1 != nil {
			s := renderParam.status1
			img, err := s.RenderIndicator()
			if err != nil {
				log.Printf("error creating image: %v\n", err)
			}
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
			}
			payload.Status1 = &imgBase64
		}

		if err := client.SetFeedback(ctx, payload); err != nil {
			log.Printf("error setting feedback: %v\n", err)
			return err
		}

	default:
		log.Printf("unknown controller: %v\n", instProps.Controller)
		return fmt.Errorf("unknown controller: %v", instProps.Controller)
	}

	return nil
}

func adjustStripGain(vm *voicemeeter.Remote, stripIndex int, delta float64) error {
	if vm == nil {
		log.Printf("vm is nil\n")
		return fmt.Errorf("vm is nil")
	}

	if stripIndex >= len(vm.Strip) || stripIndex < 0 {
		log.Printf("stripIndex %v is out of range\n", stripIndex)
		return fmt.Errorf("stripIndex %v is out of range", stripIndex)
	}

	strip := vm.Strip[stripIndex]
	gain := strip.Gain()
	gain += delta
	if gain > 12.0 {
		gain = 12.0
	}
	if gain < -60.0 {
		gain = -60.0
	}
	strip.SetGain(gain)

	return nil
}

func adjustBusGain(vm *voicemeeter.Remote, busIndex int, delta float64) error {
	if vm == nil {
		log.Printf("vm is nil\n")
		return fmt.Errorf("vm is nil")
	}

	if busIndex >= len(vm.Bus) || busIndex < 0 {
		log.Printf("busIndex %v is out of range\n", busIndex)
		return fmt.Errorf("busIndex %v is out of range", busIndex)
	}

	bus := vm.Bus[busIndex]
	gain := bus.Gain()
	gain += delta
	if gain > 12.0 {
		gain = 12.0
	}
	if gain < -60.0 {
		gain = -60.0
	}
	bus.SetGain(gain)

	return nil
}

func toggleMuteStrip(vm *voicemeeter.Remote, stripIndex int) error {
	if vm == nil {
		log.Printf("vm is nil\n")
		return fmt.Errorf("vm is nil")
	}

	if stripIndex >= len(vm.Strip) || stripIndex < 0 {
		log.Printf("stripIndex %v is out of range\n", stripIndex)
		return fmt.Errorf("stripIndex %v is out of range", stripIndex)
	}

	strip := vm.Strip[stripIndex]
	mute := strip.Mute()
	strip.SetMute(!mute)

	return nil
}

func toggleMuteBus(vm *voicemeeter.Remote, busIndex int) error {
	if vm == nil {
		log.Printf("vm is nil\n")
		return fmt.Errorf("vm is nil")
	}

	if busIndex >= len(vm.Bus) || busIndex < 0 {
		log.Printf("busIndex %v is out of range\n", busIndex)
		return fmt.Errorf("busIndex %v is out of range", busIndex)
	}

	bus := vm.Bus[busIndex]
	mute := bus.Mute()
	bus.SetMute(!mute)

	return nil
}
