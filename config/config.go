package config

import (
	"os"
	"strings"
)

type Config struct {
	Port         string
	DeepLCookies []string
	ProxyList    []string
}

var AppConfig Config

func InitConfig() {
	AppConfig = Config{
		Port:         getEnv("PORT", "9000"),
		DeepLCookies: getEnvAsSlice("DEEPL_COOKIES", []string{}),
		ProxyList:    getEnvAsSlice("PROXY_LIST", []string{}),
	}
}

func getEnv(key string, defaultValue string) string {
	if value, exists := os.LookupEnv(key); exists {
		return value
	}
	return defaultValue
}

func getEnvAsSlice(key string, defaultValue []string) []string {
	valueStr := getEnv(key, "")
	if valueStr == "" {
		return defaultValue
	}
	return strings.Split(valueStr, ",")
}
