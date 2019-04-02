package main

import (
	"fmt"
	"github.com/joho/godotenv"
	"io/ioutil"
	"os"
	"os/user"
	"time"

	"github.com/jonaylor89/gitlocalstats/scan"
	"github.com/jonaylor89/gitlocalstats/stats"
)

const (
	default_folder  = "/Repos"
	config_location = "/.config/gitlocalstats/config"
)

func getDefaultPath() string {
	usr, err := user.Current()
	if err != nil {
		panic(err)
	}

	return usr.HomeDir + default_folder
}

func getConfigPath() string {
	usr, err := user.Current()
	if err != nil {
		panic(err)
	}

	return usr.HomeDir + config_location

}

func main() {

	startingTime := time.Now().UTC()

	if _, err := os.Stat(getConfigPath()); os.IsNotExist(err) {

		default_config := []byte(fmt.Sprintf("folder=%s\nemail=example@email.com\n", getDefaultPath()))
		err := ioutil.WriteFile(getConfigPath(), default_config, 0644)
		if err != nil {
			panic("Error writing config file")
		}
	}

	err := godotenv.Load(getConfigPath())
	if err != nil {
		panic("Error loading configuration file")
	}

	folder := os.Getenv("folder")
	email := os.Getenv("email")

	repositories := scan.Scan(folder)
	stats.Stats(email, repositories)

	endingTime := time.Now().UTC()
	fmt.Println(endingTime.Sub(startingTime))

}
