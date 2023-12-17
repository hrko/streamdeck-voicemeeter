package gain_controll

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

var (
	instanceMap   *cmap.MapOf[string, instanceProperty]
	renderCh      chan *renderParams
	levelMeterMap *cmap.MapOf[string, *graphics.LevelMeter]
)

type instanceProperty streamdeck.WillAppearPayload[instanceSettings]

type instanceSettings struct {
	IconCodePoint   string                             `json:"iconCodePoint,omitempty"`
	IconFontParams  graphics.MaterialSymbolsFontParams `json:"iconFontParams,omitempty"`
	StripOrBusKind  string                             `json:"stripOrBusKind,omitempty"` // "Strip" | "Bus"
	StripOrBusIndex int                                `json:"stripOrBusIndex,omitempty"`
	GainDelta       string                             `json:"gainDelta,omitempty"`
}

type renderParams struct {
	targetContext string
	title         *string
	settings      *instanceSettings
	levels        *[]float64
	gain          *float64
	status        stripbus.IStripOrBusStatus
}

func defaultInstanceSettings() instanceSettings {
	return instanceSettings{
		IconCodePoint: "",
		IconFontParams: graphics.MaterialSymbolsFontParams{
			Style: "Rounded",
			Opsz:  "48",
			Wght:  "400",
			Fill:  "0",
			Grad:  "0",
		},
		StripOrBusKind:  "Strip",
		StripOrBusIndex: 0,
		GainDelta:       "3.0",
	}
}

func SetupPreClientRun(client *streamdeck.Client) {
	action := client.Action("jp.hrko.streamdeck.voicemeeter.gain-controll")
	instanceMap = cmap.NewOf[string, instanceProperty]() // key: context of action instance
	renderCh = make(chan *renderParams, 32)

	action.RegisterHandler(streamdeck.DidReceiveSettings, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.DidReceiveSettingsPayload[instanceSettings]
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
	action := client.Action("jp.hrko.streamdeck.voicemeeter.gain-controll")
	levelMeterMap = cmap.NewOf[string, *graphics.LevelMeter]() // key: context of action instance

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

		return nil
	})

	action.RegisterHandler(streamdeck.DialDown, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.DialDownPayload[instanceSettings]
		p.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

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
					if stripOrBusKind == "" {
						stripOrBusKind = "Strip"
					}
					stripOrBusIndex := actionProps.Settings.StripOrBusIndex

					renderParam.SetTitle(vm, stripOrBusKind, stripOrBusIndex)
					renderParam.SetLevels(vm, stripOrBusKind, stripOrBusIndex)
					renderParam.SetGain(vm, stripOrBusKind, stripOrBusIndex)
					renderParam.SetStatus(vm, stripOrBusKind, stripOrBusIndex)

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
	switch stripOrBusKind {
	case "Strip":
		stripCount := len(vm.Strip)
		if stripOrBusIndex >= stripCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return
		}
		levels := vm.Strip[stripOrBusIndex].Levels().PostFader()
		levels = levels[:2]
		p.levels = &levels

	case "Bus":
		busCount := len(vm.Bus)
		if stripOrBusIndex >= busCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return
		}
		levels := vm.Bus[stripOrBusIndex].Levels().All()
		levels = levels[:2]
		p.levels = &levels

	default:
		log.Printf("unknown stripOrBusKind: '%v'\n", stripOrBusKind)
		return
	}
}

func (p *renderParams) SetTitle(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	switch stripOrBusKind {
	case "Strip":
		stripCount := len(vm.Strip)
		if stripOrBusIndex >= stripCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return
		}
		title := vm.Strip[stripOrBusIndex].Label()
		if title == "" {
			title = fmt.Sprintf("Strip %v", stripOrBusIndex)
		}
		p.title = &title

	case "Bus":
		busCount := len(vm.Bus)
		if stripOrBusIndex >= busCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return
		}
		title := vm.Bus[stripOrBusIndex].Label()
		if title == "" {
			title = fmt.Sprintf("Bus %v", stripOrBusIndex)
		}
		p.title = &title

	default:
		log.Printf("unknown stripOrBusKind: '%v'\n", stripOrBusKind)
		return
	}
}

func (p *renderParams) SetGain(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) {
	switch stripOrBusKind {
	case "Strip":
		stripCount := len(vm.Strip)
		if stripOrBusIndex >= stripCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return
		}
		gain := vm.Strip[stripOrBusIndex].Gain()
		p.gain = &gain

	case "Bus":
		busCount := len(vm.Bus)
		if stripOrBusIndex >= busCount || stripOrBusIndex < 0 {
			log.Printf("stripOrBusIndex %v is out of range\n", stripOrBusIndex)
			return
		}
		gain := vm.Bus[stripOrBusIndex].Gain()
		p.gain = &gain

	default:
		log.Printf("unknown stripOrBusKind: '%v'\n", stripOrBusKind)
		return
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

func render(client *streamdeck.Client, renderParam *renderParams) error {
	ctx := context.Background()
	ctx = sdcontext.WithContext(ctx, renderParam.targetContext)

	instProps, ok := instanceMap.Get(renderParam.targetContext)
	if !ok {
		return fmt.Errorf("instProps has no key '%v'", renderParam.targetContext)
	}

	levelMeter, ok := levelMeterMap.Get(renderParam.targetContext)
	if !ok {
		levelMeter = graphics.NewLevelMeter(2)
		levelMeterMap.Set(renderParam.targetContext, levelMeter)
	}

	switch instProps.Controller {
	case "Encoder":
		payload := struct {
			Title      *string `json:"title,omitempty"`
			Icon       *string `json:"icon,omitempty"`
			LevelMeter *string `json:"levelMeter,omitempty"`
			GainValue  *string `json:"gainValue,omitempty"`
			GainSlider *string `json:"gainSlider,omitempty"`
			Status     *string `json:"status,omitempty"`
		}{}

		if renderParam.title != nil {
			payload.Title = renderParam.title
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
			svg, err := fontParams.RenderIconSVG(iconCodePoint, 48, 48, 0, 0, color.White, color.RGBA{0, 0, 0, 180}, color.Transparent, 1)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			imgString := streamdeck.ImageSvg(svg)
			payload.Icon = &imgString
		}
		if renderParam.levels != nil {
			levelMeter.Image.Width = 108
			levelMeter.Image.Height = 5
			levelMeter.Image.Padding.Left = 2
			levelMeter.Image.Padding.Right = 3
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
		if renderParam.gain != nil {
			str := fmt.Sprintf("%.1f dB", *renderParam.gain)
			payload.GainValue = &str

			gainFader := graphics.NewGainFader()
			gainFader.Width = 108
			gainFader.Height = 12
			img := gainFader.RenderHorizontal(*renderParam.gain)
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			payload.GainSlider = &imgBase64
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
