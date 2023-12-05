package main

import (
	"bytes"
	"encoding/json"
	"encoding/xml"
	"fmt"
	"html"
	"os"
	"path/filepath"
	"regexp"

	"github.com/tidwall/pretty"
)

// Diagram represents the structure of a diagram element in the XML.
type Diagram struct {
	ID   string `xml:"id,attr"`
	Name string `xml:"name,attr"`
	Root struct {
		Cells []Cell `xml:"mxCell"`
	} `xml:"mxGraphModel>root"`
}

// Cell represents the structure of an mxCell element in the XML.
type Cell struct {
	Vertex string `xml:"vertex,attr"`
	Value  string `xml:"value,attr"`
	Geom   struct {
		X      int `xml:"x,attr"`
		Y      int `xml:"y,attr"`
		Width  int `xml:"width,attr"`
		Height int `xml:"height,attr"`
	} `xml:"mxGeometry"`
}

func main() {
	if len(os.Args) != 3 {
		fmt.Println("Usage: go run script.go <input_xml_file> <output_directory>")
		os.Exit(1)
	}

	xmlFile := os.Args[1]
	outputDir := os.Args[2]

	if _, err := os.Stat(outputDir); os.IsNotExist(err) {
		fmt.Println("Error: The specified output path does not exist or is not a directory.")
		os.Exit(1)
	}

	convertDrawioToJson(xmlFile, outputDir)
}

func convertDrawioToJson(xmlFile, outputDir string) {
	xmlContent, err := os.ReadFile(xmlFile)
	if err != nil {
		fmt.Printf("Error reading XML file: %s\n", err)
		os.Exit(1)
	}

	var mxfile struct {
		Diagrams []Diagram `xml:"diagram"`
	}
	err = xml.Unmarshal(xmlContent, &mxfile)
	if err != nil {
		fmt.Printf("Error unmarshaling XML: %s\n", err)
		os.Exit(1)
	}

	re := regexp.MustCompile(`<[^>]+>`)
	for _, diagram := range mxfile.Diagrams {
		items := make([]map[string]interface{}, 0)
		for _, cell := range diagram.Root.Cells {
			if cell.Vertex != "1" {
				continue
			}

			cleanedValue := re.ReplaceAllString(cell.Value, "")
			decodedValue := html.UnescapeString(cleanedValue)

			var item map[string]interface{}
			if err := json.Unmarshal([]byte(decodedValue), &item); err != nil {
				fmt.Printf("Error decoding JSON for diagram '%s': %s\n", diagram.Name, err)
				continue
			}

			item["rect"] = [4]int{cell.Geom.X, cell.Geom.Y, cell.Geom.Width, cell.Geom.Height}
			items = append(items, item)
		}

		jsonData := map[string]interface{}{
			"id":    diagram.Name,
			"items": items,
		}

		outputFilePath := filepath.Join(outputDir, fmt.Sprintf("%s.json", diagram.Name))
		file, err := os.Create(outputFilePath)
		if err != nil {
			fmt.Printf("Error creating JSON file '%s': %s\n", outputFilePath, err)
			continue
		}

		var tmpBuf bytes.Buffer
		encoder := json.NewEncoder(&tmpBuf)
		encoder.SetIndent("", "  ")
		if err := encoder.Encode(jsonData); err != nil {
			fmt.Printf("Error writing JSON to buffer: %s\n", err)
		}
		file.Write(pretty.Pretty(tmpBuf.Bytes()))

		file.Close()
	}
}
