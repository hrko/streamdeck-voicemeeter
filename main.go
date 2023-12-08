package main

import (
	"context"
	"encoding/json"
	"fmt"
	"image"
	"image/color"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"time"

	"github.com/FlowingSPDG/streamdeck"
	sdcontext "github.com/FlowingSPDG/streamdeck/context"
	"github.com/fogleman/gg"
	"github.com/fufuok/cmap"
	"github.com/onyx-and-iris/voicemeeter/v2"
	"github.com/tdewolff/canvas"
	"github.com/tdewolff/canvas/renderers/rasterizer"
)

const (
	imgX   = 72
	imgY   = 72
	kindId = "potato"
)

type GlobalSettings struct {
	VoiceMeeterKind string `json:"voiceMeeterKind"`
}

type ActionInstanceSettings struct {
	ShowText       bool                  `json:"showText,omitempty"`
	IconCodePoint  string                `json:"iconCodePoint,omitempty"`
	IconFontParams MaterialSymbolsParams `json:"iconFontParams,omitempty"`
}

type ActionInstanceCoordinates struct {
	Column int `json:"column,omitempty"`
	Row    int `json:"row,omitempty"`
}

type ActionInstanceProperty struct {
	Controller      string                    `json:"controller,omitempty"` // "Keypad" | "Encoder"
	Coordinates     ActionInstanceCoordinates `json:"coordinates,omitempty"`
	IsInMultiAction bool                      `json:"isInMultiAction,omitempty"`
	Settings        ActionInstanceSettings    `json:"settings,omitempty"`
}

func main() {
	log.SetPrefix("package main: ")
	streamdeck.Log().SetOutput(os.Stderr)
	streamdeck.Log().SetPrefix("package streamdeck: ")

	ctx := context.Background()
	log.Println("Starting voicemeeter-streamdeck-plugin")
	if err := run(ctx); err != nil {
		panic(err)
	}
}

func run(ctx context.Context) error {
	params, err := streamdeck.ParseRegistrationParams(os.Args)
	if err != nil {
		return err
	}
	log.Printf("Registration params: %v", params)

	client := streamdeck.NewClient(ctx, params)
	log.Println("Client created")

	registerNoActionHandlers(client)
	registerActionHandlers(client)

	chErr := make(chan error)
	go func() {
		log.Println("Starting client")
		chErr <- client.Run(ctx)
	}()

	waitClientConnected(client)
	globalSettings, err := fetchGlobalSettings(ctx, client)
	if err != nil {
		log.Fatal(err)
	}
	log.Printf("Global settings: %v\n", globalSettings)

	vm, err := loginVoicemeeter(globalSettings.VoiceMeeterKind)
	if err != nil {
		log.Fatal(err)
	}
	defer vm.Logout()
	vm.EventAdd("ldirty")

	go actionLoop(client, vm)

	return <-chErr
}

var chGlobalSettings chan *GlobalSettings

func registerNoActionHandlers(client *streamdeck.Client) {
	chGlobalSettings = make(chan *GlobalSettings)
	client.RegisterNoActionHandler(streamdeck.DidReceiveGlobalSettings, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		payload := new(struct {
			Settings *GlobalSettings `json:"settings"`
		})
		err := json.Unmarshal(event.Payload, payload)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		select {
		case chGlobalSettings <- payload.Settings:
			log.Println("global settings received and sent to channel")
		default:
			log.Println("global settings received but no one is waiting for channel")
		}
		return nil
	})
}

func fetchGlobalSettings(ctx context.Context, client *streamdeck.Client) (*GlobalSettings, error) {
	if !client.IsConnected() {
		return nil, fmt.Errorf("client is not connected")
	}
	var gs *GlobalSettings
	eventReceived := make(chan struct{})
	defer close(eventReceived)
	go func() {
		gs = <-chGlobalSettings
		eventReceived <- struct{}{}
	}()
	ctx = sdcontext.WithContext(ctx, client.UUID())
	if err := client.GetGlobalSettings(ctx); err != nil {
		return nil, err
	}
	select {
	case <-eventReceived:
		return gs, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}

func loginVoicemeeter(kindId string) (*voicemeeter.Remote, error) {
	switch kindId {
	case "basic", "banana", "potato":
	default:
		log.Printf("unknown kindId: '%v', fallback to 'basic'\n", kindId)
		kindId = "basic"
	}
	vm, err := voicemeeter.NewRemote(kindId, 0)
	if err != nil {
		return nil, err
	}
	log.Println("Login to voicemeeter")
	err = vm.Login()
	if err != nil {
		return nil, err
	}
	return vm, nil
}

func waitClientConnected(client *streamdeck.Client) error {
	if !client.IsConnected() {
		log.Println("Waiting for client to connect")
		for !client.IsConnected() {
			time.Sleep(time.Second / 10)
		}
	}
	return nil
}

var actionInstanceMap *cmap.MapOf[string, ActionInstanceProperty]

func registerActionHandlers(client *streamdeck.Client) {
	action := client.Action("jp.hrko.voicemeeter.action")
	actionInstanceMap = cmap.NewOf[string, ActionInstanceProperty]() // key: context of action instance

	action.RegisterHandler(streamdeck.DidReceiveSettings, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		var prop ActionInstanceProperty
		err := json.Unmarshal(event.Payload, &prop)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		actionInstanceMap.Set(event.Context, prop)
		return nil
	})

	action.RegisterHandler(streamdeck.SendToPlugin, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		return nil
	})

	action.RegisterHandler(streamdeck.KeyDown, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		return nil
	})

	action.RegisterHandler(streamdeck.KeyUp, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		return nil
	})

	action.RegisterHandler(streamdeck.WillAppear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		var prop ActionInstanceProperty
		err := json.Unmarshal(event.Payload, &prop)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		actionInstanceMap.Set(event.Context, prop)
		return nil
	})

	action.RegisterHandler(streamdeck.WillDisappear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		actionInstanceMap.Remove(event.Context)
		return nil
	})
}

func actionLoop(client *streamdeck.Client, vm *voicemeeter.Remote) {
	const refreshInterval = time.Second / 30
	encoderLastIconFontParams := cmap.NewOf[string, MaterialSymbolsParams]()
	encoderLastIconCodePoint := cmap.NewOf[string, string]()
	encoderIconBase64Cache := cmap.NewOf[string, string]()
	for range time.Tick(refreshInterval) {
		for item := range actionInstanceMap.IterBuffered() {
			const busIndex = 5
			const levelMaxDb = 12.0
			const levelGoodDb = -24.0
			const levelMinDb = -60.0
			busCount := len(vm.Bus)
			if busIndex >= busCount {
				log.Printf("busIndex %v is out of range\n", busIndex)
				continue
			}
			levels := vm.Bus[busIndex].Levels().All()
			levels = levels[:2]

			ctxStr := item.Key
			ctx := context.Background()
			ctx = sdcontext.WithContext(ctx, ctxStr)

			switch item.Val.Controller {
			case "Keypad":
				img := levelMeterHorizontal(levels, levelMinDb, levelGoodDb, levelMaxDb, imgX, imgY, 2, 1, 1)
				imgBase64, err := streamdeck.Image(img)
				if err != nil {
					log.Printf("error creating image: %v\n", err)
					continue
				}
				err = client.SetImage(ctx, imgBase64, streamdeck.HardwareAndSoftware)
				if err != nil {
					log.Printf("error setting image: %v\n", err)
					continue
				}
				title := ""
				levelAvgDb := 0.0
				for _, lvDb := range levels {
					levelAvgDb += lvDb
				}
				levelAvgDb /= float64(len(levels))
				if item.Val.Settings.ShowText {
					title = fmt.Sprintf("%.1f dB", levelAvgDb)
				}

				if err := client.SetTitle(ctx, title, streamdeck.HardwareAndSoftware); err != nil {
					log.Printf("error setting title: %v\n", err)
					continue
				}

			case "Encoder":
				lastFontParams, ok := encoderLastIconFontParams.Get(ctxStr)
				if !ok {
					lastFontParams = MaterialSymbolsParams{}
				}
				encoderLastIconFontParams.Set(ctxStr, item.Val.Settings.IconFontParams)

				lastIconCodePoint, ok := encoderLastIconCodePoint.Get(ctxStr)
				if !ok {
					lastIconCodePoint = ""
				}
				encoderLastIconCodePoint.Set(ctxStr, item.Val.Settings.IconCodePoint)

				var imgIconBase64 string
				if lastFontParams != item.Val.Settings.IconFontParams || lastIconCodePoint != item.Val.Settings.IconCodePoint {
					fontParams := item.Val.Settings.IconFontParams
					fontParams.setDefaultsForEmptyParam()
					if err := fontParams.assert(); err != nil {
						log.Printf("invalid iconFontParams: %v\n", err)
						fontParams = MaterialSymbolsParams{}
						fontParams.setDefaultsForEmptyParam()
					}
					iconCodePoint := item.Val.Settings.IconCodePoint
					if iconCodePoint == "" {
						iconCodePoint = "e050" // volume_up
					}
					imgIcon, err := getMaterialSymbolsIcon(fontParams, iconCodePoint)
					if err != nil {
						log.Printf("error creating image: %v\n", err)
						continue
					}
					imgIconBase64, err = streamdeck.Image(imgIcon)
					if err != nil {
						log.Printf("error creating image: %v\n", err)
					}
					encoderIconBase64Cache.Set(ctxStr, imgIconBase64)
				} else {
					imgIconBase64, ok = encoderIconBase64Cache.Get(ctxStr)
					if !ok {
						log.Printf("iconBase64 not found in cache\n")
					}
				}

				imgLevelMeter := levelMeterHorizontal(levels, levelMinDb, levelGoodDb, levelMaxDb, 108, 8, 1, 1, 1)
				imgLevelMeterBase64, err := streamdeck.Image(imgLevelMeter)
				if err != nil {
					log.Printf("error creating image: %v\n", err)
					continue
				}
				payload := struct {
					Title      string `json:"title"`
					Icon       string `json:"icon"`
					LevelMeter string `json:"levelMeter"`
					GainValue  string `json:"gainValue"`
				}{
					Title:      vm.Bus[busIndex].Label(),
					Icon:       imgIconBase64,
					LevelMeter: imgLevelMeterBase64,
					GainValue:  fmt.Sprintf("%.1f dB", vm.Bus[busIndex].Gain()),
				}
				err = client.SetFeedback(ctx, payload)
				if err != nil {
					log.Printf("error setting feedback: %v\n", err)
					continue
				}

			default:
				log.Printf("unknown controller: %v\n", item.Val.Controller)
				continue
			}
		}
	}
}

func mmToPoints(mm float64) float64 {
	return mm * 2.834645669291339
}

func getMaterialSymbolsIcon(fontParams MaterialSymbolsParams, codePoint string) (image.Image, error) {
	fontMaterial := canvas.NewFontFamily("Material Symbols")
	fontData, err := getMaterialSymbols(fontParams)
	if err != nil {
		return nil, err
	}
	err = fontMaterial.LoadFont(fontData, 0, canvas.FontRegular)
	if err != nil {
		return nil, err
	}
	face := fontMaterial.Face(mmToPoints(48), color.White, canvas.FontRegular, canvas.FontNormal)

	c := canvas.New(48, 48)
	ctx := canvas.NewContext(c)
	codeInt, err := strconv.ParseInt(codePoint, 16, 32)
	if err != nil {
		return nil, err
	}
	codeRune := rune(codeInt)
	ctx.DrawText(0, 0, canvas.NewTextLine(face, string(codeRune), canvas.Left))

	return rasterizer.Draw(c, canvas.DPMM(1.0), canvas.DefaultColorSpace), nil
}

func levelMeterHorizontal(dB []float64, dBMin float64, dBGood float64, dBMax float64, width int, height int, cellWidth int, cellMarginX int, cellMarginY int) image.Image {
	channelCount := len(dB)
	cellHeight := (height - cellMarginY*(channelCount-1)) / channelCount
	cellCount := (width + cellMarginX) / (cellWidth + cellMarginX)
	minGoodCellIndex := int((dBGood - dBMin) / (dBMax - dBMin) * float64(cellCount))
	minClipCellIndex := int((0.0 - dBMin) / (dBMax - dBMin) * float64(cellCount))

	backgroundColor := color.RGBA{R: 0x00, G: 0x00, B: 0x00, A: 0xff}
	cellOnNormalColor := color.RGBA{R: 133, G: 173, B: 185, A: 0xff}
	cellOnGoodColor := color.RGBA{R: 30, G: 254, B: 91, A: 0xff}
	cellOnClipColor := color.RGBA{R: 250, G: 0, B: 0, A: 0xff}
	cellOffNormalColor := color.RGBA{R: 25, G: 27, B: 27, A: 0xff}
	cellOffGoodColor := color.RGBA{R: 25, G: 27, B: 27, A: 0xff}
	cellOffClipColor := color.RGBA{R: 31, G: 23, B: 21, A: 0xff}

	img := image.NewRGBA(image.Rect(0, 0, width, height))
	dc := gg.NewContextForRGBA(img)

	dc.SetColor(backgroundColor)
	dc.DrawRectangle(0, 0, float64(width), float64(height))
	dc.Fill()

	for ch, lvDb := range dB {
		lv := 0.0
		if lvDb > dBMax {
			lv = 1.0
		} else if lvDb > dBMin {
			lv = (lvDb - dBMin) / (dBMax - dBMin)
		} else {
			lv = 0.0
		}
		minOffCellIndex := int(lv * float64(cellCount))
		for i := 0; i < cellCount; i++ {
			x := i * (cellWidth + cellMarginX)
			y := ch * (cellHeight + cellMarginY)
			w := cellWidth
			h := cellHeight
			if i < minOffCellIndex {
				if i < minGoodCellIndex {
					dc.SetColor(cellOnNormalColor)
				} else if i < minClipCellIndex {
					dc.SetColor(cellOnGoodColor)
				} else {
					dc.SetColor(cellOnClipColor)
				}
			} else {
				if i < minGoodCellIndex {
					dc.SetColor(cellOffNormalColor)
				} else if i < minClipCellIndex {
					dc.SetColor(cellOffGoodColor)
				} else {
					dc.SetColor(cellOffClipColor)
				}
			}
			dc.DrawRectangle(float64(x), float64(y), float64(w), float64(h))
			dc.Fill()
		}
	}

	return img
}

type MaterialSymbolsParams struct {
	Style string `json:"style"`
	Opsz  string `json:"opsz"`
	Wght  string `json:"wght"`
	Fill  string `json:"fill"`
	Grad  string `json:"grad"`
}

func (p *MaterialSymbolsParams) String() string {
	return fmt.Sprintf("%s-%s-%s-%s-%s", p.Style, p.Opsz, p.Wght, p.Fill, p.Grad)
}

func (p *MaterialSymbolsParams) setDefaultsForEmptyParam() {
	if p.Style == "" {
		p.Style = "Outlined"
	}
	if p.Opsz == "" {
		p.Opsz = "48"
	}
	if p.Wght == "" {
		p.Wght = "400"
	}
	if p.Fill == "" {
		p.Fill = "0"
	}
	if p.Grad == "" {
		p.Grad = "0"
	}
}

func (p *MaterialSymbolsParams) assert() error {
	// style: "Outlined" | "Rounded" | "Sharp"
	// opsz: "20" | "24" | "40" | "48"
	// wght: "100" | "200" | "300" | "400" | "500" | "600" | "700"
	// fill: "0" | "1"
	// grad: "-25" | "-0" | "200"
	switch p.Style {
	case "Outlined", "Rounded", "Sharp":
	default:
		return fmt.Errorf("style must be one of 'Outlined', 'Rounded', 'Sharp'")
	}
	switch p.Opsz {
	case "20", "24", "40", "48":
	default:
		return fmt.Errorf("opsz must be one of '20', '24', '40', '48'")
	}
	switch p.Wght {
	case "100", "200", "300", "400", "500", "600", "700":
	default:
		return fmt.Errorf("wght must be one of '100', '200', '300', '400', '500', '600', '700'")
	}
	switch p.Fill {
	case "0", "1":
	default:
		return fmt.Errorf("fill must be one of '0', '1'")
	}
	switch p.Grad {
	case "-25", "0", "200":
	default:
		return fmt.Errorf("grad must be one of '-25', '0', '200'")
	}
	return nil
}

func downloadMaterialSymbolsWoff2(p MaterialSymbolsParams) ([]byte, error) {
	if err := p.assert(); err != nil {
		return nil, err
	}

	cssURL := fmt.Sprintf("https://fonts.googleapis.com/css2?family=Material+Symbols+%s:opsz,wght,FILL,GRAD@%s,%s,%s,%s", p.Style, p.Opsz, p.Wght, p.Fill, p.Grad)
	log.Println(cssURL)

	const userAgent = "	Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"
	req, err := http.NewRequest("GET", cssURL, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("User-Agent", userAgent)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	cssContent, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	re := regexp.MustCompile(`url\((https://[^)]+\.woff2)\)`)
	log.Println(string(cssContent))
	matches := re.FindStringSubmatch(string(cssContent))
	if len(matches) < 2 {
		return nil, fmt.Errorf("no woff2 file found in css")
	}

	woff2URL := matches[1]

	resp, err = http.Get(woff2URL)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	return data, nil
}

var materialSymbolsCache *cmap.MapOf[string, []byte]

func getMaterialSymbols(p MaterialSymbolsParams) ([]byte, error) {
	cacheDir, err := getCacheDir()
	if err != nil {
		return nil, err
	}
	fontCachePath := filepath.Join(cacheDir, "materialSymbolsCache.json")

	if materialSymbolsCache == nil {
		materialSymbolsCache = cmap.NewOf[string, []byte]()

		if isFileExist(fontCachePath) {
			data, err := os.ReadFile(fontCachePath)
			if err != nil {
				return nil, err
			}
			materialSymbolsCache.UnmarshalJSON(data)
		}
	}

	if err := p.assert(); err != nil {
		return nil, err
	}
	p.setDefaultsForEmptyParam()

	key := p.String()
	data, ok := materialSymbolsCache.Get(key)
	if ok {
		return data, nil
	}

	data, err = downloadMaterialSymbolsWoff2(p)
	if err != nil {
		return nil, err
	}

	materialSymbolsCache.Set(key, data)

	data, err = materialSymbolsCache.MarshalJSON()
	if err != nil {
		return nil, err
	}
	err = os.WriteFile(fontCachePath, data, 0644)
	if err != nil {
		return nil, err
	}

	return data, nil
}

func isFileExist(p string) bool {
	info, err := os.Stat(p)
	return err == nil && !info.IsDir()
}

func isDirExist(p string) bool {
	info, err := os.Stat(p)
	return err == nil && info.IsDir()
}

func getCacheDir() (string, error) {
	userCacheDir, err := os.UserCacheDir()
	if err != nil {
		return "", err
	}
	cacheDir := filepath.Join(userCacheDir, "voicemeeter-streamdeck-plugin")
	if !isDirExist(cacheDir) {
		err = os.MkdirAll(cacheDir, 0755)
		if err != nil {
			return "", err
		}
	}
	return cacheDir, nil
}
