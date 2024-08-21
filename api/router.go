package api

import (
	"net/http"

	"deeplx-pro/initialize"

	"github.com/gin-gonic/gin"
)

var router *gin.Engine

func init() {
	// 初始化gin
	router = initialize.InitRouter()
}

func Listen(w http.ResponseWriter, r *http.Request) {
	router.ServeHTTP(w, r)
}
