package main

import (
	"log"
	"net/http"
	"os"
	"time"

	"deeplx-pro/config"
	"deeplx-pro/initialize"
	"deeplx-pro/translator"

	"github.com/joho/godotenv"
)

func main() {
	// 加载环境变量
	godotenv.Load()

	// 初始化配置
	config.InitConfig()

	// 初始化翻译器
	translator.InitTranslator()

	port := config.AppConfig.Port

	// 初始化Gin引擎和路由
	r := initialize.InitRouter()

	log.Printf("Server is running on http://localhost:%s", port)
	log.Printf("DEEPL_COOKIES: %s", os.Getenv("DEEPL_COOKIES"))
	log.Printf("PROXY_LIST: %s", os.Getenv("PROXY_LIST"))

	// 设置并启动HTTP服务器
	s := &http.Server{
		Addr:           ":" + port,
		Handler:        r,
		ReadTimeout:    10 * time.Second,
		WriteTimeout:   10 * time.Second,
		MaxHeaderBytes: 1 << 20,
	}

	if err := s.ListenAndServe(); err != nil {
		log.Fatalf("Failed to run server: %v", err)
	}
}
