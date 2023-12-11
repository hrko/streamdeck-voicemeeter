package graphics

import (
	"fmt"
	"image"
	"image/color"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"strconv"

	"github.com/fufuok/cmap"
	"github.com/tdewolff/canvas"
	"github.com/tdewolff/canvas/renderers/rasterizer"
)

var (
	materialSymbolsFonts    *cmap.MapOf[string, []byte] // key: params.String()
	materialSymbolsCacheDir string
)

type MaterialSymbolsFontParams struct {
	Style string `json:"style"` // "Outlined" | "Rounded" | "Sharp"
	Opsz  string `json:"opsz"`  // "20" | "24" | "40" | "48"
	Wght  string `json:"wght"`  // "100" | "200" | "300" | "400" | "500" | "600" | "700"
	Fill  string `json:"fill"`  // "0" | "1"
	Grad  string `json:"grad"`  // "-25" | "-0" | "200"
}

// SetMaterialSymbolsCacheDir set the cache directory for Material Symbols fonts.
// If not set, the system temporary directory will be used.
// The fonts cache file is named "materialSymbolsCache.json".
func SetMaterialSymbolsCacheDir(dir string) {
	materialSymbolsCacheDir = dir
}

func (p *MaterialSymbolsFontParams) RenderIcon(codePoint string, size int) (image.Image, error) {
	font := canvas.NewFontFamily("Material Symbols")
	rawFont, err := p.getFont()
	if err != nil {
		return nil, err
	}
	err = font.LoadFont(rawFont, 0, canvas.FontRegular)
	if err != nil {
		return nil, err
	}
	sizeFloat := float64(size)
	face := font.Face(mmToPoints(sizeFloat), color.White, canvas.FontRegular, canvas.FontNormal)

	c := canvas.New(sizeFloat, sizeFloat)
	ctx := canvas.NewContext(c)
	codeInt, err := strconv.ParseInt(codePoint, 16, 32)
	if err != nil {
		return nil, err
	}
	codeRune := rune(codeInt)
	ctx.DrawText(0, 0, canvas.NewTextLine(face, string(codeRune), canvas.Left))

	return rasterizer.Draw(c, canvas.DPMM(1.0), canvas.DefaultColorSpace), nil
}

func (p *MaterialSymbolsFontParams) String() string {
	return fmt.Sprintf("%s-%s-%s-%s-%s", p.Style, p.Opsz, p.Wght, p.Fill, p.Grad)
}

func (p *MaterialSymbolsFontParams) FillEmptyWithDefault() {
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

func (p *MaterialSymbolsFontParams) Assert() error {
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

func (p *MaterialSymbolsFontParams) fetchWoff2() ([]byte, error) {
	if err := p.Assert(); err != nil {
		return nil, err
	}

	cssURL := fmt.Sprintf("https://fonts.googleapis.com/css2?family=Material+Symbols+%s:opsz,wght,FILL,GRAD@%s,%s,%s,%s", p.Style, p.Opsz, p.Wght, p.Fill, p.Grad)

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

func (p *MaterialSymbolsFontParams) getFont() ([]byte, error) {
	cacheDir := getCacheDir()
	fontsCachePath := filepath.Join(cacheDir, "materialSymbolsCache.json")

	if materialSymbolsFonts == nil {
		materialSymbolsFonts = cmap.NewOf[string, []byte]()

		if isFileExist(fontsCachePath) {
			fontsCacheJson, err := os.ReadFile(fontsCachePath)
			if err != nil {
				return nil, err
			}
			materialSymbolsFonts.UnmarshalJSON(fontsCacheJson)
		}
	}

	if err := p.Assert(); err != nil {
		return nil, err
	}
	p.FillEmptyWithDefault()

	key := p.String()
	fontData, ok := materialSymbolsFonts.Get(key)
	if ok {
		return fontData, nil
	}

	fontData, err := p.fetchWoff2()
	if err != nil {
		return nil, err
	}

	materialSymbolsFonts.Set(key, fontData)

	fontsCacheJson, err := materialSymbolsFonts.MarshalJSON()
	if err != nil {
		return nil, err
	}
	err = os.WriteFile(fontsCachePath, fontsCacheJson, 0644)
	if err != nil {
		return nil, err
	}

	return fontData, nil
}

func isFileExist(p string) bool {
	info, err := os.Stat(p)
	return err == nil && !info.IsDir()
}

func getCacheDir() string {
	if materialSymbolsCacheDir != "" {
		return materialSymbolsCacheDir
	} else {
		return os.TempDir()
	}
}

func mmToPoints(mm float64) float64 {
	return mm * 2.834645669291339
}
