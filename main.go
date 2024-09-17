package main

import (
	"flag"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/bwmarrin/discordgo"
)

var purgeTasks = make(map[string]*PurgeTask)

type PurgeTask struct {
	Interval  time.Duration
	NextPurge time.Time
}

var startTime time.Time

func main() {
	var Token string
	flag.StringVar(&Token, "t", "", "Bot Token")
	flag.Parse()

	startTime = time.Now()

	dg, err := discordgo.New("Bot " + Token)
	if err != nil {
		fmt.Println("error creating Discord session,", err)
		return
	}

	dg.AddHandler(ready)
	dg.AddHandler(applicationCommandHandler)

	err = dg.Open()
	if err != nil {
		fmt.Println("error opening connection,", err)
		return
	}

	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt, os.Kill)
	<-sc

	dg.Close()
}

func ready(s *discordgo.Session, event *discordgo.Ready) {
	s.UpdateGameStatus(0, "You look kind of familiar... have we met before?")
}

func applicationCommandHandler(s *discordgo.Session, i *discordgo.InteractionCreate) {
	if i.Type != discordgo.InteractionApplicationCommand {
		return
	}

	switch i.ApplicationCommandData().Name {
	case "set_purge_interval":
		//purge handler
	case "status":
		//status handler
	}
}
