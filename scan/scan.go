package scan

import (
	//	"fmt"
	"log"
	"os"
	"strings"
	"sync"
)

func scanGitFolders(folders *[]string, folder string, wg *sync.WaitGroup) {

	defer wg.Done()

	folder = strings.TrimSuffix(folder, "/")

	f, err := os.Open(folder)
	if err != nil {
		log.Fatal(err)
	}

	files, err := f.Readdir(-1)
	f.Close()
	if err != nil {
		log.Fatal(err)
	}

	var path string

	for _, file := range files {
		if file.IsDir() {
			path = folder + "/" + file.Name()

			// Folder is a git repo
			if file.Name() == ".git" {
				path = strings.TrimSuffix(path, "/.git")
				// Uncomment this to see what directories are being found
				// fmt.Println("[+] " + path)
				*folders = append(*folders, path)
				continue
			}

			// We really don't want to waste our time with these
			if file.Name() == "vendor" || file.Name() == "node_modules" {
				continue
			}

			// Recursively scans file systems
			wg.Add(1)
			go scanGitFolders(folders, path, wg)
		}
	}
}

func recursizeScanFolder(folder string) []string {
	folders := make([]string, 0)

	var wg sync.WaitGroup

	wg.Add(1)
	go scanGitFolders(&folders, folder, &wg)

	wg.Wait()

	return folders
}

func Scan(folder string) []string {
	// fmt.Printf("Found folder(s):\n\n")
	repositories := recursizeScanFolder(folder)

	return repositories
}
