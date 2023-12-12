package main

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
)

func main() {
	var (
		exts = []string{".json", ".exe", ".png"}
		dirs = []string{"layouts", "property_inspector"}
	)

	pluginName := os.Args[1]
	installDir := filepath.Join(getPluginsDir(), pluginName)
	if err := removeDir(installDir); err != nil {
		fmt.Printf("Error removing directory: %v\n", err)
		return
	}
	if err := makeDir(installDir); err != nil {
		fmt.Printf("Error creating directory: %v\n", err)
		return
	}

	if err := copyFiles(".", installDir, exts); err != nil {
		fmt.Printf("Error copying files: %v\n", err)
		return
	}
	if err := copyDirs(".", installDir, dirs); err != nil {
		fmt.Printf("Error copying directories: %v\n", err)
		return
	}
}

func removeDir(dir string) error {
	return os.RemoveAll(dir)
}

func makeDir(dir string) error {
	return os.Mkdir(dir, 0755)
}

func copyFiles(srcDir, destDir string, extensions []string) error {
	entries, err := os.ReadDir(srcDir)
	if err != nil {
		return err
	}
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		for _, ext := range extensions {
			if strings.HasSuffix(entry.Name(), ext) {
				if err := copyFile(filepath.Join(srcDir, entry.Name()), filepath.Join(destDir, entry.Name())); err != nil {
					return err
				}
			}
		}
	}
	return nil
}

func copyFile(src, dest string) error {
	srcFile, err := os.Open(src)
	if err != nil {
		return err
	}
	defer srcFile.Close()

	destFile, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer destFile.Close()

	_, err = io.Copy(destFile, srcFile)
	return err
}

func copyDirs(srcDir, destDir string, dirs []string) error {
	for _, dir := range dirs {
		if err := copyDir(filepath.Join(srcDir, dir), filepath.Join(destDir, dir)); err != nil {
			return err
		}
	}
	return nil
}

func copyDir(srcDir, destDir string) error {
	if err := os.MkdirAll(destDir, 0755); err != nil {
		return err
	}

	entries, err := os.ReadDir(srcDir)
	if err != nil {
		return err
	}

	for _, entry := range entries {
		srcPath := filepath.Join(srcDir, entry.Name())
		destPath := filepath.Join(destDir, entry.Name())

		if entry.IsDir() {
			if err := copyDir(srcPath, destPath); err != nil {
				return err
			}
		} else {
			if err := copyFile(srcPath, destPath); err != nil {
				return err
			}
		}
	}
	return nil
}
