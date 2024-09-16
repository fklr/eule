package main

import (
	"time"
)

// "flag"
// "fmt"
// "os"
// "os/signal"
// "syscall"
// "time"

// "github.com/bwmarrin/discordgo"

var purgeTasks = make(map[string]*PurgeTask)

type PurgeTask struct {
	Interval  time.Duration
	NextPurge time.Time
}

func main() {

}
