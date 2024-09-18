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
		handleSetPurgeInterval(s, i)
	case "status":
		handleStatusCommand(s, i)
	}
}

func handleSetPurgeInterval(s *discordgo.Session, i *discordgo.InteractionCreate) {
	options := i.ApplicationCommandData().Options
	var intervalValue int64
	var unit string
	for _, option := range options {
		switch option.Name {
		case "interval":
			intervalValue = option.IntValue()
		case "unit":
			unit = option.StringValue()
		}
	}

	var interval time.Duration
	switch unit {
	case "hours":
		interval = time.Duration(intervalValue) * time.Hour
	case "days":
		interval = time.Duration(intervalValue) * time.Hour * 24
	default:
		// Should not happen
		return
	}

	channelID := i.ChannelID
	purgeTasks[channelID] = &PurgeTask{
		Interval:  interval,
		NextPurge: time.Now().Add(interval),
	}

	response := &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Content: fmt.Sprintf("Purge interval set to %d %s for this channel.", intervalValue, unit),
		},
	}
	s.InteractionRespond(i.Interaction, response)
}

func handleStatusCommand(s *discordgo.Session, i *discordgo.InteractionCreate) {
	uptime := time.Since(startTime).Round(time.Second)
	numTasks := len(purgeTasks)
	statusMessage := fmt.Sprintf("Eule says hi!\nUptime: %s\nScheduled Purge Tasks: %d", uptime, numTasks)

	response := &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Content: statusMessage,
		},
	}
	s.InteractionRespond(i.Interaction, response)
}

func registerCommands(s *discordgo.Session) {
	commands := []*discordgo.ApplicationCommand{
		{
			Name:        "set_purge_interval",
			Description: "Set the purge interval for this channel.",
			Options: []*discordgo.ApplicationCommandOption{
				{
					Type:        discordgo.ApplicationCommandOptionInteger,
					Name:        "interval",
					Description: "Interval",
					Required:    true,
				},
				{
					Type:        discordgo.ApplicationCommandOptionString,
					Name:        "unit",
					Description: "Unit of time (hours or days)",
					Required:    true,
					Choices: []*discordgo.ApplicationCommandOptionChoice{
						{
							Name:  "Hours",
							Value: "hours",
						},
						{
							Name:  "Days",
							Value: "days",
						},
					},
				},
			},
		},
		{
			Name:        "status",
			Description: "Check Eule's status.",
		},
	}

	for _, command := range commands {
		_, err := s.ApplicationCommandCreate(s.State.User.ID, "", command)
		if err != nil {
			fmt.Printf("Cannot create '%s' command: %v\n", command.Name, err)
		}
	}
}

func purgeChecker(s *discordgo.Session) {
	ticker := time.NewTicker(1 * time.Minute)
	for {
		<-ticker.C
		now := time.Now()
		for channelID, task := range purgeTasks {
			if now.After(task.NextPurge) {
				go purgeChannel(s, channelID)
				task.NextPurge = now.Add(task.Interval)
			}
		}
	}
}

func purgeChannel(s *discordgo.Session, channelID string) {
	fmt.Printf("Purging channel %s\n", channelID)

	var messages []*discordgo.Message
	var beforeID string

	for {
		msgs, err := s.ChannelMessages(channelID, 100, beforeID, "", "")
		if err != nil {
			fmt.Println("Error getting messages:", err)
			break
		}
		if len(msgs) == 0 {
			break
		}
		messages = append(messages, msgs...)
		beforeID = msgs[len(msgs)-1].ID
		if len(msgs) < 100 {
			break
		}
	}

	for i := 0; i < len(messages); i += 100 {
		end := i + 100
		if end > len(messages) {
			end = len(messages)
		}

		messageIDs := make([]string, 0)
		for _, msg := range messages[i:end] {
			t := msg.Timestamp
			if time.Since(t) < 14*24*time.Hour {
				messageIDs = append(messageIDs, msg.ID)
			} else {
				err := s.ChannelMessageDelete(channelID, msg.ID)
				if err != nil {
					fmt.Println("Error deleting message:", err)
				}
			}
		}

		if len(messageIDs) > 1 {
			err := s.ChannelMessagesBulkDelete(channelID, messageIDs)
			if err != nil {
				fmt.Println("Error bulk deleting messages:", err)
			}
		} else if len(messageIDs) == 1 {
			err := s.ChannelMessageDelete(channelID, messageIDs[0])
			if err != nil {
				fmt.Println("Error deleting message:", err)
			}
		}
	}

	fmt.Printf("Channel %s purged.\n", channelID)
}
