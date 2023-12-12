package action

type DialRotateCommonPayload struct {
	Coordinates struct {
		Column int `json:"column"`
		Row    int `json:"row"`
	} `json:"coordinates"`
	Ticks   int  `json:"ticks"`
	Pressed bool `json:"pressed"`
}

type DialDownCommonPayload struct {
	Coordinates struct {
		Column int `json:"column"`
		Row    int `json:"row"`
	} `json:"coordinates"`
	// controller field is always "Encoder", so it's omitted here since it's useless
}

type TouchTapCommonPayload struct {
	Coordinates struct {
		Column int `json:"column"`
		Row    int `json:"row"`
	} `json:"coordinates"`
	TapPos [2]int `json:"tapPos"` // [x, y]
	Hold   bool   `json:"hold"`
}

type ActionInstanceCommonProperty struct {
	Controller  string `json:"controller,omitempty"` // "Keypad" | "Encoder"
	Coordinates struct {
		Column int `json:"column,omitempty"`
		Row    int `json:"row,omitempty"`
	} `json:"coordinates,omitempty"`
	IsInMultiAction bool `json:"isInMultiAction,omitempty"`
}
