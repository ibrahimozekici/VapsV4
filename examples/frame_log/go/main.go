package main

import (
	"context"
	"flag"
	"fmt"
	"log"

	"github.com/go-redis/redis/v8"
	"github.com/ibrahimozekici/VapsV4/api/go/stream"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
)

var (
	server string
	key    string
)

func init() {
	flag.StringVar(&server, "server", "localhost:6379", "Redis hostname:port")
	flag.StringVar(&key, "key", "gw:stream:frame", "Redis Streams key to read from")
	flag.Parse()
}

func main() {
	rdb := redis.NewClient(&redis.Options{
		Addr: server,
	})
	ctx := context.Background()

	lastID := "0"

	for {
		resp, err := rdb.XRead(ctx, &redis.XReadArgs{
			Streams: []string{key, lastID},
			Count:   10,
			Block:   0,
		}).Result()
		if err != nil {
			log.Fatal(err)
		}

		if len(resp) != 1 {
			log.Fatal("Exactly one stream response is expected")
		}

		for _, msg := range resp[0].Messages {
			lastID = msg.ID

			if b, ok := msg.Values["up"].(string); ok {
				var pl stream.UplinkFrameLog
				if err := proto.Unmarshal([]byte(b), &pl); err != nil {
					log.Fatal(err)
				}

				fmt.Println("=== UP ===")
				fmt.Println(protojson.Format(&pl))
				fmt.Println("==========")
			}

			if b, ok := msg.Values["down"].(string); ok {
				var pl stream.DownlinkFrameLog
				if err := proto.Unmarshal([]byte(b), &pl); err != nil {
					log.Fatal(err)
				}

				fmt.Println("=== DOWN ===")
				fmt.Println(protojson.Format(&pl))
				fmt.Println("============")
			}
		}
	}
}
