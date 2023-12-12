package stripbus

import (
	"fmt"
	"image"
	"image/color"
	"log"

	"github.com/onyx-and-iris/voicemeeter/v2"

	"github.com/hrko/streamdeck-voicemeeter/pkg/graphics"
)

type IStripOrBusStatus interface {
	RenderIndicator() (image.Image, error)
}

type StripStatus struct {
	VmKind     string
	IsPhysical bool
	OutPhysBus []bool
	OutVirtBus []bool
	Mute       bool
	Solo       bool
	Mono       bool
	Eq         bool
	Mc         bool
}

type BusStatus struct {
	VmKind string
	Mute   bool
	Eq     bool
	Mono   bool
}

func GetStripOrBusStatus(vm *voicemeeter.Remote, stripOrBusKind string, stripOrBusIndex int) (IStripOrBusStatus, error) {
	switch stripOrBusKind {
	case "Strip":
		return GetStripStatus(vm, stripOrBusIndex)
	case "Bus":
		return GetBusStatus(vm, stripOrBusIndex)
	default:
		return nil, fmt.Errorf("unknown stripOrBusKind %s", stripOrBusKind)
	}
}

func GetStripStatus(vm *voicemeeter.Remote, stripIndex int) (*StripStatus, error) {
	ss := &StripStatus{}

	if stripIndex < 0 || stripIndex >= len(vm.Strip) {
		log.Printf("stripIndex %v is out of range\n", stripIndex)
		return nil, fmt.Errorf("stripIndex %v is out of range", stripIndex)
	}

	ss.VmKind = vm.Kind.Name
	if stripIndex < vm.Kind.PhysIn {
		ss.IsPhysical = true
	} else {
		ss.IsPhysical = false
	}

	strip := vm.Strip[stripIndex]
	ss.Mute = strip.Mute()
	ss.Solo = strip.Solo()
	ss.OutPhysBus = make([]bool, vm.Kind.PhysOut)
	ss.OutVirtBus = make([]bool, vm.Kind.VirtOut)
	switch vm.Kind.Name {
	case "basic":
		ss.OutPhysBus[0] = strip.A1()
		ss.OutVirtBus[0] = strip.B1()
		if ss.IsPhysical {
			ss.Mono = strip.Mono()
		}

	case "banana":
		ss.OutPhysBus[0] = strip.A1()
		ss.OutPhysBus[1] = strip.A2()
		ss.OutPhysBus[2] = strip.A3()
		ss.OutVirtBus[0] = strip.B1()
		ss.OutVirtBus[1] = strip.B2()
		if ss.IsPhysical {
			ss.Mono = strip.Mono()
		} else {
			ss.Mc = strip.Mc()
		}

	case "potato":
		ss.OutPhysBus[0] = strip.A1()
		ss.OutPhysBus[1] = strip.A2()
		ss.OutPhysBus[2] = strip.A3()
		ss.OutPhysBus[3] = strip.A4()
		ss.OutPhysBus[4] = strip.A5()
		ss.OutVirtBus[0] = strip.B1()
		ss.OutVirtBus[1] = strip.B2()
		ss.OutVirtBus[2] = strip.B3()
		if ss.IsPhysical {
			ss.Mono = strip.Mono()
			ss.Eq = strip.Eq().On()
		} else {
			ss.Mc = strip.Mc()
		}
	}

	return ss, nil
}

func GetBusStatus(vm *voicemeeter.Remote, busIndex int) (*BusStatus, error) {
	bs := &BusStatus{}

	if busIndex < 0 || busIndex >= len(vm.Bus) {
		log.Printf("busIndex %v is out of range\n", busIndex)
		return nil, fmt.Errorf("busIndex %v is out of range", busIndex)
	}

	bs.VmKind = vm.Kind.Name

	bus := vm.Bus[busIndex]
	bs.Mute = bus.Mute()
	switch vm.Kind.Name {
	case "banana", "potato":
		bs.Eq = bus.Eq().On()
		bs.Mono = bus.Mono()
	}

	return bs, nil
}

func (ss *StripStatus) RenderIndicator() (image.Image, error) {
	var (
		s *graphics.StatusIndicator
	)

	switch ss.VmKind {
	case "basic":
		if ss.IsPhysical {
			s = newBasicPhysStripStatusIndicator()
		} else {
			s = newBasicVirtStripStatusIndicator()
		}
	case "banana":
		if ss.IsPhysical {
			s = newBananaPhysStripStatusIndicator()
		} else {
			s = newBananaVirtStripStatusIndicator()
		}
	case "potato":
		if ss.IsPhysical {
			s = newPotatoPhysStripStatusIndicator()
		} else {
			s = newPotatoVirtStripStatusIndicator()
		}
	default:
		return nil, fmt.Errorf("unknown kind %s", ss.VmKind)
	}

	flags := make([][]bool, len(s.Rows))

	for i := range s.Rows {
		flags[i] = make([]bool, 0, len(s.Rows[i].ColorsTrue))
	}

	switch ss.VmKind {
	case "basic":
		flags[0] = append(flags[0], ss.OutPhysBus...)
		flags[0] = append(flags[0], ss.OutVirtBus...)
		if ss.IsPhysical {
			flags[1] = append(flags[1], ss.Mute)
			flags[1] = append(flags[1], ss.Solo)
			flags[1] = append(flags[1], ss.Mono)
		} else {
			flags[1] = append(flags[1], ss.Mute)
			flags[1] = append(flags[1], ss.Solo)
		}

	case "banana":
		flags[0] = append(flags[0], ss.OutPhysBus...)
		flags[0] = append(flags[0], ss.OutVirtBus...)
		if ss.IsPhysical {
			flags[1] = append(flags[1], ss.Mute)
			flags[1] = append(flags[1], ss.Solo)
			flags[1] = append(flags[1], ss.Mono)
		} else {
			flags[1] = append(flags[1], ss.Mute)
			flags[1] = append(flags[1], ss.Solo)
			flags[1] = append(flags[1], ss.Mc)
		}

	case "potato":
		flags[0] = append(flags[0], ss.OutPhysBus...)
		flags[1] = append(flags[1], ss.OutVirtBus...)
		if ss.IsPhysical {
			flags[2] = append(flags[2], ss.Mute)
			flags[2] = append(flags[2], ss.Solo)
			flags[2] = append(flags[2], ss.Mono)
			flags[2] = append(flags[2], ss.Eq)
		} else {
			flags[2] = append(flags[2], ss.Mute)
			flags[2] = append(flags[2], ss.Solo)
			flags[2] = append(flags[2], ss.Mc)
		}
	}

	return s.Render(flags)
}

func newPotatoPhysStripStatusIndicator() *graphics.StatusIndicator {
	s := &graphics.StatusIndicator{}

	cInactive := color.RGBA{0x2c, 0x3d, 0x4d, 0xff}
	cBus := color.RGBA{0x70, 0xc3, 0x99, 0xff}
	cMute := color.RGBA{0xf6, 0x60, 0x51, 0xff}
	cSolo := color.RGBA{0xe8, 0xb1, 0x5f, 0xff}
	cMono := color.RGBA{0x68, 0xe6, 0xf8, 0xff}
	cEq := color.RGBA{0x29, 0x6f, 0xfd, 0xff}

	s.Width = 36
	s.Height = 24
	s.Rows = []graphics.StatusIndicatorRowStyle{
		{
			ColorsTrue:  []color.Color{cBus, cBus, cBus, cBus, cBus},
			ColorsFalse: []color.Color{cInactive, cInactive, cInactive, cInactive, cInactive},
			Shape:       graphics.StatusIndicatorShapeCircle,
			ItemMargin:  2.0,
			ItemSize:    5.0,
			MarginTop:   0.0,
			MarginLeft:  2.0,
			MarginRight: 0.0,
			Rtl:         false,
		},
		{
			ColorsTrue:  []color.Color{cBus, cBus, cBus},
			ColorsFalse: []color.Color{cInactive, cInactive, cInactive},
			Shape:       graphics.StatusIndicatorShapeCircle,
			ItemMargin:  2.0,
			ItemSize:    5.0,
			MarginTop:   2.0,
			MarginLeft:  2.0,
			MarginRight: 0.0,
			Rtl:         false,
		},
		{
			ColorsTrue:       []color.Color{cMute, cSolo, cMono, cEq},
			ColorsFalse:      []color.Color{cInactive, cInactive, cInactive, cInactive},
			Shape:            graphics.StatusIndicatorShapeSquare,
			ItemMargin:       2.0,
			ItemSize:         7.0,
			ItemCornerRadius: 1.5,
			MarginTop:        3.0,
			MarginLeft:       2.0,
			MarginRight:      0.0,
			Rtl:              false,
		},
	}

	return s
}

func newPotatoVirtStripStatusIndicator() *graphics.StatusIndicator {
	s := newPotatoPhysStripStatusIndicator()

	cInactive := color.RGBA{0x2c, 0x3d, 0x4d, 0xff}
	cMute := color.RGBA{0xf6, 0x60, 0x51, 0xff}
	cSolo := color.RGBA{0xe8, 0xb1, 0x5f, 0xff}
	cMc := cMute

	s.Rows[2].ColorsTrue = []color.Color{cMute, cSolo, cMc}
	s.Rows[2].ColorsFalse = []color.Color{cInactive, cInactive, cInactive}

	return s
}

func newBananaPhysStripStatusIndicator() *graphics.StatusIndicator {
	s := newPotatoPhysStripStatusIndicator()

	rows := append([]graphics.StatusIndicatorRowStyle{}, s.Rows[0], s.Rows[2])
	rows[0].MarginTop += s.Rows[1].ItemSize + s.Rows[1].MarginTop
	s.Rows = rows

	return s
}

func newBananaVirtStripStatusIndicator() *graphics.StatusIndicator {
	s := newPotatoVirtStripStatusIndicator()

	rows := append([]graphics.StatusIndicatorRowStyle{}, s.Rows[0], s.Rows[2])
	rows[0].MarginTop += s.Rows[1].ItemSize + s.Rows[1].MarginTop
	s.Rows = rows

	return s
}

func newBasicPhysStripStatusIndicator() *graphics.StatusIndicator {
	return newBananaPhysStripStatusIndicator()
}

func newBasicVirtStripStatusIndicator() *graphics.StatusIndicator {
	return newBananaVirtStripStatusIndicator()
}

func (bs *BusStatus) RenderIndicator() (image.Image, error) {
	var (
		s *graphics.StatusIndicator
	)

	switch bs.VmKind {
	case "basic":
		s = newBasicBusStatusIndicator()
	case "banana":
		s = newBananaBusStatusIndicator()
	case "potato":
		s = newPotatoBusStatusIndicator()
	default:
		return nil, fmt.Errorf("unknown kind %s", bs.VmKind)
	}

	flags := make([][]bool, len(s.Rows))

	for i := range s.Rows {
		flags[i] = make([]bool, 0, len(s.Rows[i].ColorsTrue))
	}

	switch bs.VmKind {
	case "basic":
		flags[0] = append(flags[0], bs.Mute)

	case "banana":
		flags[0] = append(flags[0], bs.Mute)
		flags[0] = append(flags[0], bs.Eq)
		flags[0] = append(flags[0], bs.Mono)

	case "potato":
		flags[0] = append(flags[0], bs.Mute)
		flags[0] = append(flags[0], bs.Eq)
		flags[0] = append(flags[0], bs.Mono)
	}

	return s.Render(flags)
}

func newPotatoBusStatusIndicator() *graphics.StatusIndicator {
	s := &graphics.StatusIndicator{}

	cInactive := color.RGBA{0x2c, 0x3d, 0x4d, 0xff}
	cMute := color.RGBA{0xf6, 0x60, 0x51, 0xff}
	cEq := color.RGBA{0x29, 0x6f, 0xfd, 0xff}
	cMono := color.RGBA{0x68, 0xe6, 0xf8, 0xff}

	s.Width = 36
	s.Height = 24
	s.Rows = []graphics.StatusIndicatorRowStyle{
		{
			ColorsTrue:       []color.Color{cMute, cEq, cMono},
			ColorsFalse:      []color.Color{cInactive, cInactive, cInactive},
			Shape:            graphics.StatusIndicatorShapeSquare,
			ItemMargin:       2.0,
			ItemSize:         7.0,
			ItemCornerRadius: 1.5,
			MarginTop:        15.0,
			MarginLeft:       2.0,
			MarginRight:      0.0,
			Rtl:              false,
		},
	}

	return s
}

func newBananaBusStatusIndicator() *graphics.StatusIndicator {
	return newPotatoBusStatusIndicator()
}

func newBasicBusStatusIndicator() *graphics.StatusIndicator {
	return newPotatoBusStatusIndicator()
}
